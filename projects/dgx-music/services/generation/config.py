"""
Configuration Management for DGX Music
======================================

Centralized configuration using Pydantic Settings.
Supports environment variables and .env files.
"""

import os
from pathlib import Path
from typing import Optional

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """
    Application settings with environment variable support.

    Environment variables override defaults:
    - DGX_MUSIC_OUTPUT_DIR
    - DGX_MUSIC_MODEL_NAME
    - DGX_MUSIC_USE_GPU
    etc.
    """

    # Application
    app_name: str = "DGX Music"
    version: str = "0.1.0-alpha"
    debug: bool = False

    # Paths
    base_dir: Path = Path(__file__).parent.parent.parent
    output_dir: Path = base_dir / "data" / "outputs"
    model_cache_dir: Path = base_dir / "data" / "models"
    log_dir: Path = base_dir / "data" / "logs"
    db_path: Path = base_dir / "data" / "generations.db"

    # Model Configuration
    model_name: str = "musicgen-small"
    use_gpu: bool = True
    gpu_device_id: int = 0

    # Generation Parameters
    default_duration: float = 16.0
    default_temperature: float = 1.0
    default_top_k: int = 250
    default_top_p: float = 0.0
    default_cfg_coef: float = 3.0
    max_duration: float = 30.0
    min_duration: float = 1.0

    # Audio Settings
    sample_rate: int = 32000
    channels: int = 2  # Stereo
    output_format: str = "wav"
    bit_depth: int = 16

    # Audio Processing
    normalize_audio: bool = True
    target_lufs: float = -16.0  # EBU R128 recommendation
    enable_limiter: bool = True
    limiter_threshold: float = -1.0

    # Performance
    max_concurrent_jobs: int = 1  # Sequential processing for MVP
    enable_model_caching: bool = True  # Keep model in memory
    enable_result_caching: bool = False  # Cache generated audio
    cache_ttl_seconds: int = 3600

    # Retry Logic (Week 3)
    max_retries: int = 3  # Maximum retry attempts for failed generations
    retry_delay_seconds: float = 1.0  # Initial retry delay (exponential backoff)

    # Memory Management
    max_memory_gb: float = 30.0  # Peak memory budget
    unload_model_after_idle_seconds: Optional[int] = None  # Keep loaded by default

    # API Settings
    api_host: str = "0.0.0.0"
    api_port: int = 8000
    api_workers: int = 1
    api_reload: bool = False  # Development only

    # Rate Limiting (Week 3)
    rate_limit_enabled: bool = True
    rate_limit_per_minute: int = 10  # Requests per minute per IP
    rate_limit_whitelist: list[str] = ["127.0.0.1", "localhost", "::1"]

    # Database
    db_echo: bool = False  # SQLAlchemy echo SQL statements

    # Logging
    log_level: str = "INFO"
    log_format: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    log_to_file: bool = True
    log_to_console: bool = True

    # Monitoring
    enable_metrics: bool = True
    metrics_port: int = 9090

    # Security (for future use)
    enable_auth: bool = False
    api_key: Optional[str] = None

    model_config = SettingsConfigDict(
        env_prefix="DGX_MUSIC_",
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore"  # Ignore extra environment variables
    )

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # Ensure directories exist
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.model_cache_dir.mkdir(parents=True, exist_ok=True)
        self.log_dir.mkdir(parents=True, exist_ok=True)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)

    @property
    def database_url(self) -> str:
        """SQLAlchemy database URL."""
        return f"sqlite:///{self.db_path}"

    @property
    def cuda_device(self) -> str:
        """CUDA device string."""
        return f"cuda:{self.gpu_device_id}"

    def get_output_path(self, job_id: str) -> Path:
        """Get output file path for a job."""
        return self.output_dir / f"{job_id}.{self.output_format}"

    def validate_cuda(self) -> bool:
        """
        Validate CUDA availability.

        Returns:
            True if CUDA is available and should be used, False otherwise.
        """
        if not self.use_gpu:
            return False

        try:
            import torch
            return torch.cuda.is_available()
        except ImportError:
            return False


# Global settings instance
settings = Settings()


def get_settings() -> Settings:
    """
    Get settings instance.

    This function is useful for FastAPI dependency injection.
    """
    return settings


def print_settings():
    """Print current settings (for debugging)."""
    print("="*60)
    print("DGX MUSIC - CONFIGURATION")
    print("="*60)
    print(f"App Name: {settings.app_name}")
    print(f"Version: {settings.version}")
    print(f"Debug Mode: {settings.debug}")
    print()
    print("PATHS:")
    print(f"  Base Dir: {settings.base_dir}")
    print(f"  Output Dir: {settings.output_dir}")
    print(f"  Model Cache: {settings.model_cache_dir}")
    print(f"  Log Dir: {settings.log_dir}")
    print(f"  Database: {settings.db_path}")
    print()
    print("MODEL:")
    print(f"  Model Name: {settings.model_name}")
    print(f"  Use GPU: {settings.use_gpu}")
    print(f"  GPU Device: {settings.gpu_device_id}")
    print()
    print("AUDIO:")
    print(f"  Sample Rate: {settings.sample_rate} Hz")
    print(f"  Channels: {settings.channels}")
    print(f"  Format: {settings.output_format}")
    print(f"  Normalize: {settings.normalize_audio}")
    print(f"  Target LUFS: {settings.target_lufs}")
    print()
    print("PERFORMANCE:")
    print(f"  Max Concurrent Jobs: {settings.max_concurrent_jobs}")
    print(f"  Model Caching: {settings.enable_model_caching}")
    print(f"  Max Memory: {settings.max_memory_gb} GB")
    print(f"  Max Retries: {settings.max_retries}")
    print(f"  Retry Delay: {settings.retry_delay_seconds}s")
    print()
    print("API:")
    print(f"  Host: {settings.api_host}")
    print(f"  Port: {settings.api_port}")
    print(f"  Workers: {settings.api_workers}")
    print(f"  Rate Limit: {settings.rate_limit_per_minute}/min")
    print()
    print("LOGGING:")
    print(f"  Level: {settings.log_level}")
    print(f"  To File: {settings.log_to_file}")
    print(f"  To Console: {settings.log_to_console}")
    print("="*60)


if __name__ == "__main__":
    print_settings()
