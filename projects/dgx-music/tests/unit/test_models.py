"""
Unit tests for database models.

Tests the Generation and Prompt ORM models without requiring
a database connection.
"""

import json
from datetime import datetime
import pytest

from services.storage.models import Generation, Prompt
from services.storage.schema import GenerationStatus


class TestGeneration:
    """Tests for the Generation model."""

    def test_generation_creation(self):
        """Test creating a Generation instance."""
        gen = Generation(
            prompt="test prompt",
            model_name="musicgen-small",
            duration_seconds=16.0,
            sample_rate=32000,
            channels=2,
            file_path="outputs/test.wav",
        )

        assert gen.prompt == "test prompt"
        assert gen.model_name == "musicgen-small"
        assert gen.duration_seconds == 16.0
        assert gen.sample_rate == 32000
        assert gen.channels == 2
        assert gen.file_path == "outputs/test.wav"
        assert gen.status == GenerationStatus.PENDING

    def test_generation_default_values(self):
        """Test that Generation has proper default values."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        assert gen.status == GenerationStatus.PENDING
        assert gen.file_size_bytes is None
        assert gen.completed_at is None
        assert gen.generation_time_seconds is None
        assert gen.error_message is None

    def test_generation_uuid_id(self):
        """Test that Generation ID is a UUID."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        # UUID should be a string
        assert isinstance(gen.id, str)
        # UUID v4 format: 8-4-4-4-12 characters
        assert len(gen.id) == 36
        assert gen.id.count("-") == 4

    def test_is_pending(self):
        """Test is_pending property."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
            status=GenerationStatus.PENDING,
        )

        assert gen.is_pending is True
        assert gen.is_processing is False
        assert gen.is_complete is False
        assert gen.is_failed is False
        assert gen.is_finished is False

    def test_is_processing(self):
        """Test is_processing property."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
            status=GenerationStatus.PROCESSING,
        )

        assert gen.is_pending is False
        assert gen.is_processing is True
        assert gen.is_complete is False
        assert gen.is_failed is False
        assert gen.is_finished is False

    def test_is_complete(self):
        """Test is_complete property."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
            status=GenerationStatus.COMPLETED,
        )

        assert gen.is_pending is False
        assert gen.is_processing is False
        assert gen.is_complete is True
        assert gen.is_failed is False
        assert gen.is_finished is True

    def test_is_failed(self):
        """Test is_failed property."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
            status=GenerationStatus.FAILED,
        )

        assert gen.is_pending is False
        assert gen.is_processing is False
        assert gen.is_complete is False
        assert gen.is_failed is True
        assert gen.is_finished is True

    def test_get_metadata_empty(self):
        """Test get_metadata with no metadata."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        assert gen.get_metadata() == {}

    def test_set_and_get_metadata(self):
        """Test setting and getting metadata."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        metadata = {"bpm": 120, "key": "C", "genre": "hip hop"}
        gen.set_metadata(metadata)

        retrieved = gen.get_metadata()
        assert retrieved == metadata
        assert retrieved["bpm"] == 120
        assert retrieved["key"] == "C"
        assert retrieved["genre"] == "hip hop"

    def test_metadata_json_serialization(self):
        """Test that metadata is stored as JSON string."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        metadata = {"bpm": 120, "key": "C"}
        gen.set_metadata(metadata)

        # Verify it's stored as JSON string
        assert isinstance(gen.metadata, str)
        assert json.loads(gen.metadata) == metadata

    def test_mark_processing(self):
        """Test mark_processing method."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        gen.mark_processing()
        assert gen.status == GenerationStatus.PROCESSING
        assert gen.is_processing is True

    def test_mark_completed(self):
        """Test mark_completed method."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        gen.mark_completed(generation_time=18.5)

        assert gen.status == GenerationStatus.COMPLETED
        assert gen.is_complete is True
        assert gen.generation_time_seconds == 18.5
        assert gen.completed_at is not None
        assert isinstance(gen.completed_at, datetime)

    def test_mark_failed(self):
        """Test mark_failed method."""
        gen = Generation(
            prompt="test",
            model_name="test",
            duration_seconds=10.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        error_msg = "Model loading failed"
        gen.mark_failed(error_msg)

        assert gen.status == GenerationStatus.FAILED
        assert gen.is_failed is True
        assert gen.error_message == error_msg
        assert gen.completed_at is not None

    def test_to_dict(self):
        """Test to_dict method."""
        gen = Generation(
            prompt="test prompt",
            model_name="musicgen-small",
            model_version="1.0",
            duration_seconds=16.0,
            sample_rate=32000,
            channels=2,
            file_path="outputs/test.wav",
        )

        gen.set_metadata({"bpm": 140})

        result = gen.to_dict()

        assert result["prompt"] == "test prompt"
        assert result["model_name"] == "musicgen-small"
        assert result["model_version"] == "1.0"
        assert result["duration_seconds"] == 16.0
        assert result["sample_rate"] == 32000
        assert result["channels"] == 2
        assert result["file_path"] == "outputs/test.wav"
        assert result["status"] == GenerationStatus.PENDING
        assert result["metadata"] == {"bpm": 140}

    def test_repr(self):
        """Test string representation."""
        gen = Generation(
            prompt="test prompt for hip hop beat",
            model_name="musicgen-small",
            duration_seconds=16.0,
            sample_rate=32000,
            channels=2,
            file_path="test.wav",
        )

        repr_str = repr(gen)
        assert "Generation" in repr_str
        assert gen.status in repr_str


class TestPrompt:
    """Tests for the Prompt model."""

    def test_prompt_creation(self):
        """Test creating a Prompt instance."""
        prompt = Prompt(text="test prompt")

        assert prompt.text == "test prompt"
        assert prompt.used_count == 1
        assert prompt.first_used_at is not None
        assert prompt.last_used_at is not None

    def test_prompt_default_values(self):
        """Test Prompt default values."""
        prompt = Prompt(text="test")

        assert prompt.used_count == 1
        assert isinstance(prompt.first_used_at, datetime)
        assert isinstance(prompt.last_used_at, datetime)

    def test_increment_usage(self):
        """Test increment_usage method."""
        prompt = Prompt(text="test")

        original_last_used = prompt.last_used_at
        original_count = prompt.used_count

        # Increment usage
        prompt.increment_usage()

        assert prompt.used_count == original_count + 1
        assert prompt.last_used_at >= original_last_used

    def test_increment_usage_multiple_times(self):
        """Test incrementing usage multiple times."""
        prompt = Prompt(text="test")

        for i in range(5):
            prompt.increment_usage()

        assert prompt.used_count == 6  # 1 initial + 5 increments

    def test_to_dict(self):
        """Test to_dict method."""
        prompt = Prompt(text="test prompt")

        result = prompt.to_dict()

        assert result["text"] == "test prompt"
        assert result["used_count"] == 1
        assert "first_used_at" in result
        assert "last_used_at" in result

    def test_repr(self):
        """Test string representation."""
        prompt = Prompt(text="test prompt for representation")

        repr_str = repr(prompt)
        assert "Prompt" in repr_str
        assert str(prompt.used_count) in repr_str
