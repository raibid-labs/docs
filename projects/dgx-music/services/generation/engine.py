"""
Core Music Generation Engine
============================

MusicGen wrapper with error handling, performance monitoring, and memory management.
Implements the critical path for MVP - Week 1 deliverable.
"""

import time
import traceback
from pathlib import Path
from typing import Optional, Tuple
from uuid import uuid4

import torch
import numpy as np
import soundfile as sf

from .config import settings
from .logger import get_logger, LogContext, log_performance, log_memory_usage
from .models import (
    GenerationRequest,
    GenerationResult,
    GenerationStatus,
    AudioMetadata,
    PerformanceBenchmark,
)


logger = get_logger("engine")


class GenerationError(Exception):
    """Base exception for generation errors."""
    pass


class ModelLoadError(GenerationError):
    """Error loading the AI model."""
    pass


class GenerationTimeoutError(GenerationError):
    """Generation exceeded time limit."""
    pass


class MusicGenerationEngine:
    """
    Core music generation engine using MusicGen.

    This class handles:
    - Model loading and caching
    - Prompt encoding
    - Audio generation
    - Error handling and recovery
    - Performance monitoring
    - Memory management
    """

    def __init__(
        self,
        model_name: str = "small",
        use_gpu: bool = True,
        enable_caching: bool = True,
    ):
        """
        Initialize the generation engine.

        Args:
            model_name: MusicGen model variant (small, medium, large)
            use_gpu: Use GPU if available
            enable_caching: Keep model in memory between generations
        """
        self.model_name = model_name
        self.use_gpu = use_gpu and self._check_cuda()
        self.enable_caching = enable_caching
        self.model = None
        self.device = "cuda" if self.use_gpu else "cpu"
        self._generation_count = 0
        self._total_generation_time = 0.0

        logger.info(
            f"Initializing MusicGenerationEngine: "
            f"model={model_name} device={self.device} caching={enable_caching}"
        )

        # Load model immediately if caching enabled
        if self.enable_caching:
            self.load_model()

    def _check_cuda(self) -> bool:
        """Check if CUDA is available."""
        try:
            import torch
            available = torch.cuda.is_available()
            if available:
                logger.info(f"CUDA available: {torch.cuda.get_device_name(0)}")
            else:
                logger.warning("CUDA not available, using CPU (will be slower)")
            return available
        except ImportError:
            logger.error("PyTorch not installed")
            return False

    def load_model(self) -> None:
        """
        Load MusicGen model into memory.

        Raises:
            ModelLoadError: If model fails to load
        """
        if self.model is not None:
            logger.debug("Model already loaded")
            return

        try:
            logger.info(f"Loading MusicGen model: {self.model_name}")
            start_time = time.time()

            from audiocraft.models import MusicGen

            # Load pretrained model
            self.model = MusicGen.get_pretrained(self.model_name)

            # Move to appropriate device
            if self.use_gpu:
                self.model = self.model.to(self.device)

            load_time = time.time() - start_time
            logger.info(f"Model loaded successfully in {load_time:.2f}s")
            log_memory_usage()

        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            logger.error(traceback.format_exc())
            raise ModelLoadError(f"Could not load model {self.model_name}: {e}")

    def unload_model(self) -> None:
        """Unload model from memory to free resources."""
        if self.model is not None:
            logger.info("Unloading model from memory")
            self.model = None

            # Clear GPU cache if using CUDA
            if self.use_gpu and torch.cuda.is_available():
                torch.cuda.empty_cache()

            log_memory_usage()

    def set_generation_params(
        self,
        duration: float = 16.0,
        temperature: float = 1.0,
        top_k: int = 250,
        top_p: float = 0.0,
        cfg_coef: float = 3.0,
    ) -> None:
        """
        Set generation parameters.

        Args:
            duration: Audio duration in seconds
            temperature: Sampling temperature (higher = more random)
            top_k: Top-k sampling
            top_p: Nucleus sampling
            cfg_coef: Classifier-free guidance coefficient
        """
        if self.model is None:
            self.load_model()

        self.model.set_generation_params(
            duration=duration,
            temperature=temperature,
            top_k=top_k,
            top_p=top_p,
            cfg_coef=cfg_coef,
        )

        logger.debug(
            f"Generation params set: duration={duration}s temp={temperature} "
            f"top_k={top_k} top_p={top_p} cfg_coef={cfg_coef}"
        )

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
        Generate audio from text prompt.

        Args:
            prompt: Text description of desired music
            duration: Audio duration in seconds
            temperature: Sampling temperature
            top_k: Top-k sampling
            top_p: Nucleus sampling
            cfg_coef: Classifier-free guidance

        Returns:
            Tuple of (audio_array, sample_rate)

        Raises:
            GenerationError: If generation fails
        """
        job_id = str(uuid4())[:8]

        with LogContext(job_id=job_id, operation="generate"):
            logger.info(f"Generating audio: prompt='{prompt}' duration={duration}s")

            try:
                # Ensure model is loaded
                if self.model is None:
                    self.load_model()

                # Set generation parameters
                self.set_generation_params(
                    duration=duration,
                    temperature=temperature,
                    top_k=top_k,
                    top_p=top_p,
                    cfg_coef=cfg_coef,
                )

                # Generate
                start_time = time.time()
                log_memory_usage()

                with torch.no_grad():  # Disable gradient computation
                    wav = self.model.generate([prompt])  # Batch of 1

                generation_time = time.time() - start_time

                # Extract audio data
                audio = wav[0].cpu().numpy()  # Shape: (channels, samples)
                sample_rate = self.model.sample_rate

                # Update statistics
                self._generation_count += 1
                self._total_generation_time += generation_time

                # Log performance
                log_performance(
                    "audio_generation",
                    generation_time,
                    success=True,
                    duration=duration,
                    sample_rate=sample_rate,
                    real_time_factor=generation_time / duration,
                )

                log_memory_usage()

                logger.info(
                    f"Generation complete: {generation_time:.2f}s "
                    f"(RTF: {generation_time/duration:.2f}x)"
                )

                return audio, sample_rate

            except Exception as e:
                logger.error(f"Generation failed: {e}")
                logger.error(traceback.format_exc())
                raise GenerationError(f"Audio generation failed: {e}")

    def save_audio(
        self,
        audio: np.ndarray,
        sample_rate: int,
        output_path: Path,
        normalize: bool = True,
    ) -> AudioMetadata:
        """
        Save generated audio to file.

        Args:
            audio: Audio array (channels, samples)
            sample_rate: Sample rate in Hz
            output_path: Output file path
            normalize: Apply loudness normalization

        Returns:
            Audio metadata

        Raises:
            GenerationError: If save fails
        """
        try:
            logger.debug(f"Saving audio to {output_path}")

            # Ensure output directory exists
            output_path.parent.mkdir(parents=True, exist_ok=True)

            # Transpose for soundfile (samples, channels)
            audio_t = audio.T

            # Normalize if requested
            if normalize and settings.normalize_audio:
                audio_t = self._normalize_loudness(audio_t, sample_rate)

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

            logger.info(
                f"Audio saved: {output_path.name} "
                f"({metadata.file_size_mb:.2f}MB, {duration:.1f}s)"
            )

            return metadata

        except Exception as e:
            logger.error(f"Failed to save audio: {e}")
            raise GenerationError(f"Could not save audio: {e}")

    def _normalize_loudness(
        self,
        audio: np.ndarray,
        sample_rate: int,
        target_lufs: float = -16.0,
    ) -> np.ndarray:
        """
        Normalize audio loudness to target LUFS.

        Args:
            audio: Audio array (samples, channels)
            sample_rate: Sample rate
            target_lufs: Target loudness in LUFS

        Returns:
            Normalized audio
        """
        try:
            import pyloudnorm as pyln

            # Measure loudness
            meter = pyln.Meter(sample_rate)
            loudness = meter.integrated_loudness(audio)

            # Normalize
            normalized = pyln.normalize.loudness(audio, loudness, target_lufs)

            logger.debug(f"Normalized audio: {loudness:.1f} -> {target_lufs:.1f} LUFS")

            return normalized

        except ImportError:
            logger.warning("pyloudnorm not available, skipping normalization")
            return audio
        except Exception as e:
            logger.warning(f"Normalization failed: {e}, using original audio")
            return audio

    def generate(self, request: GenerationRequest) -> GenerationResult:
        """
        High-level generation function that handles the complete workflow.

        Args:
            request: Generation request with prompt and parameters

        Returns:
            Generation result with file path and metadata
        """
        job_id = f"gen_{uuid4().hex[:8]}"
        start_time = time.time()

        with LogContext(job_id=job_id):
            logger.info(f"Starting generation job: {job_id}")

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
                output_path = settings.get_output_path(job_id)
                metadata = self.save_audio(
                    audio=audio,
                    sample_rate=sample_rate,
                    output_path=output_path,
                    normalize=settings.normalize_audio,
                )

                # Calculate total time
                total_time = time.time() - start_time

                # Build result
                result = GenerationResult(
                    job_id=job_id,
                    status=GenerationStatus.COMPLETED,
                    prompt=request.prompt,
                    model=f"musicgen-{self.model_name}",
                    file_url=f"/api/v1/files/{output_path.name}",
                    file_path=str(output_path),
                    metadata=metadata,
                    generation_time_seconds=total_time,
                    created_at=start_time,
                    completed_at=time.time(),
                )

                logger.info(f"Job completed: {job_id} in {total_time:.2f}s")

                return result

            except Exception as e:
                logger.error(f"Job failed: {job_id} - {e}")

                result = GenerationResult(
                    job_id=job_id,
                    status=GenerationStatus.FAILED,
                    prompt=request.prompt,
                    model=f"musicgen-{self.model_name}",
                    error_message=str(e),
                    created_at=start_time,
                )

                return result

    def benchmark(self, duration: float = 16.0) -> PerformanceBenchmark:
        """
        Run a performance benchmark.

        Args:
            duration: Test duration in seconds

        Returns:
            Benchmark results
        """
        logger.info(f"Running benchmark: duration={duration}s")

        # Test prompt
        prompt = "upbeat electronic dance music with synth melody"

        try:
            # Generate
            start_time = time.time()
            audio, sample_rate = self.generate_audio(prompt, duration=duration)
            generation_time = time.time() - start_time

            # Measure memory
            peak_memory_mb = 0.0
            gpu_util = None

            if self.use_gpu and torch.cuda.is_available():
                peak_memory_mb = torch.cuda.max_memory_allocated() / (1024 * 1024)
                # Reset peak memory counter
                torch.cuda.reset_peak_memory_stats()

            # Build benchmark result
            benchmark = PerformanceBenchmark(
                model_name=f"musicgen-{self.model_name}",
                duration=duration,
                generation_time=generation_time,
                real_time_factor=generation_time / duration,
                peak_memory_mb=peak_memory_mb,
                gpu_utilization_percent=gpu_util,
                sample_rate=sample_rate,
                cuda_available=self.use_gpu,
            )

            logger.info(
                f"Benchmark complete: {generation_time:.2f}s "
                f"(RTF: {benchmark.real_time_factor:.2f}x) "
                f"Peak memory: {peak_memory_mb:.1f}MB"
            )

            return benchmark

        except Exception as e:
            logger.error(f"Benchmark failed: {e}")
            raise

    def get_stats(self) -> dict:
        """Get engine statistics."""
        return {
            "model_name": f"musicgen-{self.model_name}",
            "device": self.device,
            "model_loaded": self.model is not None,
            "generation_count": self._generation_count,
            "total_generation_time": self._total_generation_time,
            "average_generation_time": (
                self._total_generation_time / self._generation_count
                if self._generation_count > 0
                else 0.0
            ),
        }


# Global engine instance (singleton pattern)
_engine: Optional[MusicGenerationEngine] = None


def get_engine() -> MusicGenerationEngine:
    """
    Get or create the global engine instance.

    Returns:
        MusicGenerationEngine instance
    """
    global _engine
    if _engine is None:
        _engine = MusicGenerationEngine(
            model_name=settings.model_name.replace("musicgen-", ""),
            use_gpu=settings.use_gpu,
            enable_caching=settings.enable_model_caching,
        )
    return _engine


if __name__ == "__main__":
    # Test the engine
    print("Testing MusicGenerationEngine...")

    engine = get_engine()
    print(f"Engine initialized: {engine.get_stats()}")

    # Run benchmark
    print("\nRunning benchmark...")
    benchmark = engine.benchmark(duration=8.0)
    print(f"Benchmark results: {benchmark.model_dump_json(indent=2)}")
