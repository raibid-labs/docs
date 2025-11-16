"""
Audio processing services for DGX Music.

This package provides audio export, metadata extraction, and file management
for generated music files.

Public API:
    AudioExporter: Export PyTorch tensors to WAV files with normalization
    AudioMetadataExtractor: Extract metadata (BPM, duration, etc.) from audio
    AudioFileManager: Manage audio file storage and organization

Usage:
    from services.audio import AudioExporter, AudioFileManager

    # Export audio
    exporter = AudioExporter(target_lufs=-16.0)
    output_path, file_size = exporter.export_wav(
        audio_tensor=tensor,
        output_path="output.wav",
        sample_rate=32000
    )

    # Manage files
    file_manager = AudioFileManager()
    path = file_manager.get_output_path(job_id)
"""

from .export import AudioExporter
from .metadata import AudioMetadataExtractor
from .storage import AudioFileManager

__all__ = [
    "AudioExporter",
    "AudioMetadataExtractor",
    "AudioFileManager",
]

__version__ = "1.0.0"
