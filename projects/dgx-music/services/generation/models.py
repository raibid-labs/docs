"""
Pydantic Models for Music Generation Service
============================================

Data models for requests, responses, and internal state management.
"""

from datetime import datetime
from enum import Enum
from typing import Optional, Dict, Any, List
from uuid import UUID, uuid4

from pydantic import BaseModel, Field, field_validator


class GenerationStatus(str, Enum):
    """Status of a generation job."""
    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class GenerationStep(str, Enum):
    """Current step in the generation pipeline."""
    QUEUED = "queued"
    LOADING_MODEL = "loading_model"
    ENCODING_PROMPT = "encoding_prompt"
    GENERATING = "generating"
    SAVING = "saving"
    COMPLETED = "completed"
    FAILED = "failed"


class ModelName(str, Enum):
    """Supported AI models."""
    MUSICGEN_SMALL = "musicgen-small"
    MUSICGEN_MEDIUM = "musicgen-medium"
    MUSICGEN_LARGE = "musicgen-large"


class GenerationRequest(BaseModel):
    """Request to generate music from a text prompt."""

    prompt: str = Field(
        ...,
        min_length=3,
        max_length=500,
        description="Text description of the desired music"
    )
    duration: float = Field(
        default=16.0,
        ge=1.0,
        le=30.0,
        description="Duration in seconds (1-30s for MVP)"
    )
    temperature: float = Field(
        default=1.0,
        ge=0.1,
        le=2.0,
        description="Sampling temperature (0.1-2.0, higher = more random)"
    )
    top_k: int = Field(
        default=250,
        ge=0,
        le=500,
        description="Top-k sampling parameter"
    )
    top_p: float = Field(
        default=0.0,
        ge=0.0,
        le=1.0,
        description="Top-p (nucleus) sampling parameter"
    )
    cfg_coef: float = Field(
        default=3.0,
        ge=1.0,
        le=10.0,
        description="Classifier-free guidance coefficient"
    )
    model: ModelName = Field(
        default=ModelName.MUSICGEN_SMALL,
        description="Model to use for generation"
    )

    @field_validator("prompt")
    @classmethod
    def validate_prompt(cls, v: str) -> str:
        """Validate and clean the prompt."""
        v = v.strip()
        if not v:
            raise ValueError("Prompt cannot be empty")
        return v

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "prompt": "trap beat with heavy 808 bass and sharp hi-hats at 140 BPM",
                    "duration": 16.0,
                    "temperature": 1.0,
                    "top_k": 250,
                    "top_p": 0.0,
                    "cfg_coef": 3.0,
                    "model": "musicgen-small"
                }
            ]
        }
    }


class BatchGenerationRequest(BaseModel):
    """Request to generate multiple music tracks in batch."""

    requests: List[GenerationRequest] = Field(
        ...,
        min_length=1,
        max_length=10,
        description="List of generation requests (max 10)"
    )

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "requests": [
                        {
                            "prompt": "upbeat electronic dance music",
                            "duration": 16.0
                        },
                        {
                            "prompt": "calm piano melody",
                            "duration": 20.0
                        }
                    ]
                }
            ]
        }
    }


class BatchGenerationResponse(BaseModel):
    """Response after submitting a batch generation request."""

    job_ids: List[str] = Field(..., description="List of job identifiers")
    total_jobs: int = Field(..., description="Total number of jobs submitted")
    estimated_total_time_seconds: float = Field(..., description="Estimated total completion time")

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "job_ids": ["gen_a1b2c3d4", "gen_e5f6g7h8"],
                    "total_jobs": 2,
                    "estimated_total_time_seconds": 40.0
                }
            ]
        }
    }


class GenerationResponse(BaseModel):
    """Response after submitting a generation request."""

    job_id: str = Field(..., description="Unique job identifier")
    status: GenerationStatus = Field(..., description="Current job status")
    current_step: GenerationStep = Field(default=GenerationStep.QUEUED, description="Current pipeline step")
    estimated_time_seconds: float = Field(..., description="Estimated completion time")
    created_at: datetime = Field(default_factory=datetime.utcnow)

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "job_id": "gen_a1b2c3d4",
                    "status": "pending",
                    "current_step": "queued",
                    "estimated_time_seconds": 20.0,
                    "created_at": "2025-11-07T10:30:00Z"
                }
            ]
        }
    }


class AudioMetadata(BaseModel):
    """Metadata about generated audio."""

    duration: float = Field(..., description="Duration in seconds")
    sample_rate: int = Field(..., description="Sample rate in Hz")
    channels: int = Field(..., description="Number of audio channels")
    file_size_bytes: int = Field(..., description="File size in bytes")
    file_size_mb: float = Field(..., description="File size in MB")
    format: str = Field(default="wav", description="Audio format")


class GenerationResult(BaseModel):
    """Complete result of a generation job."""

    job_id: str = Field(..., description="Unique job identifier")
    status: GenerationStatus = Field(..., description="Final job status")
    current_step: GenerationStep = Field(default=GenerationStep.QUEUED, description="Current pipeline step")
    prompt: str = Field(..., description="Original prompt")
    model: str = Field(..., description="Model used")
    file_url: Optional[str] = Field(None, description="Download URL for audio file")
    file_path: Optional[str] = Field(None, description="Local file path")
    metadata: Optional[AudioMetadata] = Field(None, description="Audio metadata")
    generation_time_seconds: Optional[float] = Field(None, description="Time taken to generate")
    error_message: Optional[str] = Field(None, description="Error message if failed")
    retry_count: Optional[int] = Field(0, description="Number of retry attempts")
    created_at: datetime = Field(..., description="Creation timestamp")
    completed_at: Optional[datetime] = Field(None, description="Completion timestamp")

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "job_id": "gen_a1b2c3d4",
                    "status": "completed",
                    "current_step": "completed",
                    "prompt": "trap beat with heavy 808 bass...",
                    "model": "musicgen-small",
                    "file_url": "/api/v1/files/gen_a1b2c3d4.wav",
                    "file_path": "/opt/dgx-music/outputs/gen_a1b2c3d4.wav",
                    "metadata": {
                        "duration": 16.0,
                        "sample_rate": 32000,
                        "channels": 2,
                        "file_size_bytes": 3145728,
                        "file_size_mb": 3.0,
                        "format": "wav"
                    },
                    "generation_time_seconds": 18.4,
                    "retry_count": 0,
                    "created_at": "2025-11-07T10:30:00Z",
                    "completed_at": "2025-11-07T10:30:18Z"
                }
            ]
        }
    }


class GenerationHistory(BaseModel):
    """List of generation jobs."""

    generations: list[GenerationResult] = Field(default_factory=list)
    total: int = Field(..., description="Total number of generations")
    page: int = Field(default=1, ge=1)
    page_size: int = Field(default=20, ge=1, le=100)


class HealthCheck(BaseModel):
    """Individual health check result."""

    name: str = Field(..., description="Check name")
    status: str = Field(..., description="Check status: healthy, degraded, unhealthy")
    message: Optional[str] = Field(None, description="Additional information")
    details: Optional[Dict[str, Any]] = Field(None, description="Detailed check data")


class HealthStatus(BaseModel):
    """Health check response."""

    status: str = Field(..., description="Overall health status: healthy, degraded, unhealthy")
    checks: Dict[str, HealthCheck] = Field(..., description="Individual health checks")
    version: str = Field(..., description="Service version")
    uptime_seconds: float = Field(..., description="Service uptime")
    timestamp: datetime = Field(default_factory=datetime.utcnow)

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "status": "healthy",
                    "checks": {
                        "database": {
                            "name": "database",
                            "status": "healthy",
                            "message": "Connected"
                        },
                        "gpu": {
                            "name": "gpu",
                            "status": "healthy",
                            "message": "NVIDIA A100 available",
                            "details": {"device_count": 1}
                        }
                    },
                    "version": "0.1.0-alpha",
                    "uptime_seconds": 3600.5,
                    "timestamp": "2025-11-07T10:30:00Z"
                }
            ]
        }
    }


class GenerationConfig(BaseModel):
    """Configuration for the generation engine."""

    model_name: ModelName = Field(default=ModelName.MUSICGEN_SMALL)
    use_gpu: bool = Field(default=True, description="Use GPU if available")
    output_dir: str = Field(default="data/outputs")
    sample_rate: int = Field(default=32000, description="Output sample rate")
    channels: int = Field(default=2, description="Output channels (1=mono, 2=stereo)")
    format: str = Field(default="wav", description="Output format")
    normalize_audio: bool = Field(default=True, description="Apply loudness normalization")
    target_lufs: float = Field(default=-16.0, description="Target loudness in LUFS")
    max_concurrent_jobs: int = Field(default=1, description="Max parallel generations")
    enable_caching: bool = Field(default=False, description="Cache generated audio")
    max_retries: int = Field(default=3, description="Maximum retry attempts for failed generations")
    retry_delay_seconds: float = Field(default=1.0, description="Initial retry delay (exponential backoff)")


class PerformanceBenchmark(BaseModel):
    """Performance benchmark results."""

    model_name: str
    duration: float
    generation_time: float
    real_time_factor: float = Field(..., description="generation_time / duration")
    peak_memory_mb: float
    gpu_utilization_percent: Optional[float] = None
    sample_rate: int
    cuda_available: bool
    timestamp: datetime = Field(default_factory=datetime.utcnow)

    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "model_name": "musicgen-small",
                    "duration": 16.0,
                    "generation_time": 18.4,
                    "real_time_factor": 1.15,
                    "peak_memory_mb": 8500,
                    "gpu_utilization_percent": 85.0,
                    "sample_rate": 32000,
                    "cuda_available": True,
                    "timestamp": "2025-11-07T10:30:00Z"
                }
            ]
        }
    }


class QueueStats(BaseModel):
    """Queue statistics."""

    pending_jobs: int = Field(..., description="Number of pending jobs")
    processing_jobs: int = Field(..., description="Number of jobs currently processing")
    completed_jobs: int = Field(..., description="Number of completed jobs")
    failed_jobs: int = Field(..., description="Number of failed jobs")
    oldest_pending_job_age_seconds: Optional[float] = Field(None, description="Age of oldest pending job")
    average_processing_time_seconds: Optional[float] = Field(None, description="Average processing time")
