"""
DGX Music - Generation Service
==============================

Core music generation service using MusicGen.
"""

from .engine import (
    MusicGenerationEngine,
    get_engine,
    GenerationError,
    ModelLoadError,
)
from .models import (
    GenerationRequest,
    GenerationResponse,
    GenerationResult,
    GenerationStatus,
    AudioMetadata,
    ModelName,
    PerformanceBenchmark,
)
from .config import settings, get_settings
from .logger import get_logger

__all__ = [
    # Engine
    "MusicGenerationEngine",
    "get_engine",
    "GenerationError",
    "ModelLoadError",
    # Models
    "GenerationRequest",
    "GenerationResponse",
    "GenerationResult",
    "GenerationStatus",
    "AudioMetadata",
    "ModelName",
    "PerformanceBenchmark",
    # Config
    "settings",
    "get_settings",
    # Logger
    "get_logger",
]

__version__ = "0.1.0-alpha"
