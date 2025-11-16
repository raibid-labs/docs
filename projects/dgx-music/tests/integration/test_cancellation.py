"""
Integration Tests for Job Cancellation
======================================

Tests for job cancellation functionality through the API.
"""

import pytest
from fastapi.testclient import TestClient
from unittest.mock import Mock, patch
from datetime import datetime

from services.generation.api import app
from services.generation.models import (
    GenerationResult,
    GenerationStatus,
    GenerationStep,
)


@pytest.fixture
def client():
    """Create test client."""
    return TestClient(app)


@pytest.fixture
def mock_service():
    """Create mock service."""
    service = Mock()
    return service


class TestJobCancellation:
    """Test cases for job cancellation."""

    def test_cancel_pending_job_succeeds(self, client, mock_service):
        """Test that cancelling a pending job succeeds."""
        # Mock job status as pending
        mock_service.get_job_status.return_value = GenerationResult(
            job_id="gen_test123",
            status=GenerationStatus.PENDING,
            current_step=GenerationStep.QUEUED,
            prompt="test prompt",
            model="musicgen-small",
            created_at=datetime.utcnow(),
        )
        mock_service.cancel_job.return_value = True

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/gen_test123")

        assert response.status_code == 200
        data = response.json()
        assert data["job_id"] == "gen_test123"
        assert "cancelled_at" in data

    def test_cancel_processing_job_fails(self, client, mock_service):
        """Test that cancelling a processing job fails."""
        # Mock job status as processing
        mock_service.get_job_status.return_value = GenerationResult(
            job_id="gen_test123",
            status=GenerationStatus.PROCESSING,
            current_step=GenerationStep.GENERATING,
            prompt="test prompt",
            model="musicgen-small",
            created_at=datetime.utcnow(),
        )

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/gen_test123")

        assert response.status_code == 409  # Conflict
        data = response.json()
        assert "currently processing" in data["detail"].lower()

    def test_cancel_completed_job_fails(self, client, mock_service):
        """Test that cancelling a completed job fails."""
        # Mock job status as completed
        mock_service.get_job_status.return_value = GenerationResult(
            job_id="gen_test123",
            status=GenerationStatus.COMPLETED,
            current_step=GenerationStep.COMPLETED,
            prompt="test prompt",
            model="musicgen-small",
            created_at=datetime.utcnow(),
            completed_at=datetime.utcnow(),
        )

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/gen_test123")

        assert response.status_code == 409  # Conflict

    def test_cancel_failed_job_fails(self, client, mock_service):
        """Test that cancelling a failed job fails."""
        # Mock job status as failed
        mock_service.get_job_status.return_value = GenerationResult(
            job_id="gen_test123",
            status=GenerationStatus.FAILED,
            current_step=GenerationStep.FAILED,
            prompt="test prompt",
            model="musicgen-small",
            error_message="Generation failed",
            created_at=datetime.utcnow(),
            completed_at=datetime.utcnow(),
        )

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/gen_test123")

        assert response.status_code == 409  # Conflict

    def test_cancel_nonexistent_job_fails(self, client, mock_service):
        """Test that cancelling a non-existent job fails."""
        mock_service.get_job_status.return_value = None

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/nonexistent")

        assert response.status_code == 404  # Not found

    def test_cancel_job_updates_database(self, client, mock_service):
        """Test that cancelling a job updates the database."""
        mock_service.get_job_status.return_value = GenerationResult(
            job_id="gen_test123",
            status=GenerationStatus.PENDING,
            current_step=GenerationStep.QUEUED,
            prompt="test prompt",
            model="musicgen-small",
            created_at=datetime.utcnow(),
        )
        mock_service.cancel_job.return_value = True

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/gen_test123")

        assert response.status_code == 200
        # Verify that cancel_job was called
        mock_service.cancel_job.assert_called_once_with("gen_test123")

    def test_cancel_job_removes_from_queue(self, client, mock_service):
        """Test that cancelling a job removes it from the queue."""
        mock_service.get_job_status.return_value = GenerationResult(
            job_id="gen_test123",
            status=GenerationStatus.PENDING,
            current_step=GenerationStep.QUEUED,
            prompt="test prompt",
            model="musicgen-small",
            created_at=datetime.utcnow(),
        )
        mock_service.cancel_job.return_value = True

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/gen_test123")

        assert response.status_code == 200
        mock_service.cancel_job.assert_called_once()

    def test_cancel_returns_proper_error_message(self, client, mock_service):
        """Test that cancel returns proper error messages."""
        mock_service.get_job_status.return_value = GenerationResult(
            job_id="gen_test123",
            status=GenerationStatus.PROCESSING,
            current_step=GenerationStep.GENERATING,
            prompt="test prompt",
            model="musicgen-small",
            created_at=datetime.utcnow(),
        )

        with patch("services.generation.api.service", mock_service):
            response = client.delete("/api/v1/jobs/gen_test123")

        data = response.json()
        assert "detail" in data
        assert isinstance(data["detail"], str)
