"""
SQLAlchemy ORM models for DGX Music.

This module defines the database models using SQLAlchemy 2.0 ORM patterns.
Models correspond to the schema defined in schema.py.

Key Models:
- Generation: Music generation jobs and results
- Prompt: Unique prompts with usage tracking
"""

from datetime import datetime
from typing import Any, Dict, Optional
import uuid
import json

from sqlalchemy import Column, String, Float, Integer, DateTime, Text, Index
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.sql import func

from .schema import GenerationStatus, validate_status

Base = declarative_base()


class Generation(Base):
    """
    Model representing a music generation job.

    Tracks the full lifecycle of a generation request from creation
    through processing to completion or failure.

    Attributes:
        id: Unique UUID identifier
        prompt: Text prompt describing desired music
        model_name: AI model used for generation
        model_version: Specific model version/checkpoint
        duration_seconds: Target audio duration
        sample_rate: Audio sample rate (Hz)
        channels: Number of audio channels (1=mono, 2=stereo)
        file_path: Path to generated WAV file
        file_size_bytes: Size of generated file
        status: Current job status (pending/processing/completed/failed)
        created_at: Timestamp of job creation
        completed_at: Timestamp of job completion
        generation_time_seconds: Time taken to generate audio
        error_message: Error details if job failed
        metadata: JSON metadata (BPM, key, tempo, etc.)
    """

    __tablename__ = "generations"

    id = Column(String, primary_key=True, default=lambda: str(uuid.uuid4()))
    prompt = Column(Text, nullable=False)
    model_name = Column(String, nullable=False)
    model_version = Column(String, nullable=True)
    duration_seconds = Column(Float, nullable=False)
    sample_rate = Column(Integer, nullable=False)
    channels = Column(Integer, nullable=False)
    file_path = Column(String, nullable=False)
    file_size_bytes = Column(Integer, nullable=True)
    status = Column(String, nullable=False, default=GenerationStatus.PENDING)
    created_at = Column(DateTime, nullable=False, default=datetime.utcnow)
    completed_at = Column(DateTime, nullable=True)
    generation_time_seconds = Column(Float, nullable=True)
    error_message = Column(Text, nullable=True)
    metadata = Column(Text, nullable=True)  # JSON stored as text

    # Indexes
    __table_args__ = (
        Index("idx_generations_status", "status"),
        Index("idx_generations_created_at", "created_at"),
        Index("idx_generations_model_name", "model_name"),
        Index("idx_generations_completed_at", "completed_at"),
    )

    def __repr__(self) -> str:
        """String representation of Generation."""
        return (
            f"<Generation(id={self.id[:8]}..., "
            f"status={self.status}, "
            f"prompt='{self.prompt[:30]}...')>"
        )

    @property
    def is_pending(self) -> bool:
        """Check if generation is pending."""
        return self.status == GenerationStatus.PENDING

    @property
    def is_processing(self) -> bool:
        """Check if generation is currently processing."""
        return self.status == GenerationStatus.PROCESSING

    @property
    def is_complete(self) -> bool:
        """Check if generation completed successfully."""
        return self.status == GenerationStatus.COMPLETED

    @property
    def is_failed(self) -> bool:
        """Check if generation failed."""
        return self.status == GenerationStatus.FAILED

    @property
    def is_finished(self) -> bool:
        """Check if generation is in a terminal state (completed or failed)."""
        return self.is_complete or self.is_failed

    def get_metadata(self) -> Dict[str, Any]:
        """
        Parse and return metadata as a dictionary.

        Returns:
            Dictionary of metadata, or empty dict if none
        """
        if self.metadata is None:
            return {}
        try:
            return json.loads(self.metadata)
        except (json.JSONDecodeError, TypeError):
            return {}

    def set_metadata(self, metadata_dict: Dict[str, Any]) -> None:
        """
        Set metadata from a dictionary.

        Args:
            metadata_dict: Dictionary to serialize as JSON
        """
        self.metadata = json.dumps(metadata_dict)

    def mark_processing(self) -> None:
        """Mark generation as processing."""
        self.status = GenerationStatus.PROCESSING

    def mark_completed(self, generation_time: float) -> None:
        """
        Mark generation as completed.

        Args:
            generation_time: Time taken to generate (seconds)
        """
        self.status = GenerationStatus.COMPLETED
        self.completed_at = datetime.utcnow()
        self.generation_time_seconds = generation_time

    def mark_failed(self, error_message: str) -> None:
        """
        Mark generation as failed.

        Args:
            error_message: Description of the failure
        """
        self.status = GenerationStatus.FAILED
        self.completed_at = datetime.utcnow()
        self.error_message = error_message

    def to_dict(self) -> Dict[str, Any]:
        """
        Convert model to dictionary for API responses.

        Returns:
            Dictionary representation of the model
        """
        return {
            "id": self.id,
            "prompt": self.prompt,
            "model_name": self.model_name,
            "model_version": self.model_version,
            "duration_seconds": self.duration_seconds,
            "sample_rate": self.sample_rate,
            "channels": self.channels,
            "file_path": self.file_path,
            "file_size_bytes": self.file_size_bytes,
            "status": self.status,
            "created_at": self.created_at.isoformat() if self.created_at else None,
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "generation_time_seconds": self.generation_time_seconds,
            "error_message": self.error_message,
            "metadata": self.get_metadata(),
        }


class Prompt(Base):
    """
    Model for tracking unique prompts and their usage.

    This table helps with analytics and prompt history tracking.

    Attributes:
        id: Auto-incrementing primary key
        text: Unique prompt text
        used_count: Number of times this prompt was used
        first_used_at: Timestamp of first use
        last_used_at: Timestamp of most recent use
    """

    __tablename__ = "prompts"

    id = Column(Integer, primary_key=True, autoincrement=True)
    text = Column(Text, nullable=False, unique=True)
    used_count = Column(Integer, nullable=False, default=1)
    first_used_at = Column(DateTime, nullable=False, default=datetime.utcnow)
    last_used_at = Column(DateTime, nullable=False, default=datetime.utcnow)

    # Index
    __table_args__ = (Index("idx_prompts_text", "text"),)

    def __repr__(self) -> str:
        """String representation of Prompt."""
        return (
            f"<Prompt(id={self.id}, "
            f"text='{self.text[:30]}...', "
            f"used_count={self.used_count})>"
        )

    def increment_usage(self) -> None:
        """Increment usage count and update last used timestamp."""
        self.used_count += 1
        self.last_used_at = datetime.utcnow()

    def to_dict(self) -> Dict[str, Any]:
        """
        Convert model to dictionary for API responses.

        Returns:
            Dictionary representation of the model
        """
        return {
            "id": self.id,
            "text": self.text,
            "used_count": self.used_count,
            "first_used_at": self.first_used_at.isoformat() if self.first_used_at else None,
            "last_used_at": self.last_used_at.isoformat() if self.last_used_at else None,
        }
