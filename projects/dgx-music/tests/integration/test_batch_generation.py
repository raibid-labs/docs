"""
Integration Tests for Batch Generation
======================================

Tests for batch generation functionality through the API.
"""

import pytest
from fastapi.testclient import TestClient
from unittest.mock import Mock, patch

from services.generation.api import app
from services.generation.models import GenerationRequest


@pytest.fixture
def client():
    """Create test client."""
    return TestClient(app)


@pytest.fixture
def mock_service():
    """Create mock service."""
    service = Mock()
    service.submit_batch = Mock(return_value=["gen_abc123", "gen_def456"])
    service._estimate_generation_time = Mock(return_value=20.0)
    return service


class TestBatchGeneration:
    """Test cases for batch generation."""

    def test_batch_endpoint_accepts_multiple_requests(self, client, mock_service):
        """Test that batch endpoint accepts multiple requests."""
        with patch("services.generation.api.service", mock_service):
            response = client.post(
                "/api/v1/generate/batch",
                json={
                    "requests": [
                        {"prompt": "upbeat electronic music"},
                        {"prompt": "calm piano melody"},
                    ]
                },
            )

        assert response.status_code == 202
        data = response.json()
        assert "job_ids" in data
        assert len(data["job_ids"]) == 2

    def test_batch_response_includes_all_job_ids(self, client, mock_service):
        """Test that batch response includes all job IDs."""
        with patch("services.generation.api.service", mock_service):
            response = client.post(
                "/api/v1/generate/batch",
                json={
                    "requests": [
                        {"prompt": "prompt 1"},
                        {"prompt": "prompt 2"},
                        {"prompt": "prompt 3"},
                    ]
                },
            )

        data = response.json()
        assert data["total_jobs"] == 2  # Mock returns 2 IDs
        assert len(data["job_ids"]) == 2

    def test_batch_response_includes_estimated_time(self, client, mock_service):
        """Test that batch response includes total estimated time."""
        with patch("services.generation.api.service", mock_service):
            response = client.post(
                "/api/v1/generate/batch",
                json={
                    "requests": [
                        {"prompt": "prompt 1", "duration": 16.0},
                        {"prompt": "prompt 2", "duration": 20.0},
                    ]
                },
            )

        data = response.json()
        assert "estimated_total_time_seconds" in data
        assert data["estimated_total_time_seconds"] > 0

    def test_batch_rejects_empty_request_list(self, client):
        """Test that batch endpoint rejects empty request list."""
        response = client.post(
            "/api/v1/generate/batch",
            json={"requests": []},
        )

        assert response.status_code == 422  # Validation error

    def test_batch_rejects_too_many_requests(self, client):
        """Test that batch endpoint rejects more than 10 requests."""
        requests = [{"prompt": f"prompt {i}"} for i in range(11)]

        response = client.post(
            "/api/v1/generate/batch",
            json={"requests": requests},
        )

        assert response.status_code == 422  # Validation error

    def test_batch_accepts_maximum_allowed_requests(self, client, mock_service):
        """Test that batch endpoint accepts exactly 10 requests."""
        requests = [{"prompt": f"prompt {i}"} for i in range(10)]

        with patch("services.generation.api.service", mock_service):
            response = client.post(
                "/api/v1/generate/batch",
                json={"requests": requests},
            )

        assert response.status_code == 202

    def test_batch_validates_individual_requests(self, client):
        """Test that batch validates each individual request."""
        response = client.post(
            "/api/v1/generate/batch",
            json={
                "requests": [
                    {"prompt": "valid prompt"},
                    {"prompt": ""},  # Invalid: empty prompt
                ]
            },
        )

        assert response.status_code == 422  # Validation error

    def test_batch_preserves_request_parameters(self, client, mock_service):
        """Test that batch preserves all request parameters."""
        with patch("services.generation.api.service", mock_service):
            response = client.post(
                "/api/v1/generate/batch",
                json={
                    "requests": [
                        {
                            "prompt": "prompt 1",
                            "duration": 20.0,
                            "temperature": 1.5,
                            "top_k": 200,
                        },
                        {
                            "prompt": "prompt 2",
                            "duration": 15.0,
                            "temperature": 0.8,
                            "top_k": 300,
                        },
                    ]
                },
            )

        assert response.status_code == 202

        # Check that service was called with correct requests
        call_args = mock_service.submit_batch.call_args[0][0]
        assert call_args[0].prompt == "prompt 1"
        assert call_args[0].duration == 20.0
        assert call_args[0].temperature == 1.5
        assert call_args[1].prompt == "prompt 2"
        assert call_args[1].duration == 15.0

    def test_batch_uses_defaults_for_missing_parameters(self, client, mock_service):
        """Test that batch uses default values for missing parameters."""
        with patch("services.generation.api.service", mock_service):
            response = client.post(
                "/api/v1/generate/batch",
                json={
                    "requests": [
                        {"prompt": "prompt 1"},  # Only prompt specified
                    ]
                },
            )

        assert response.status_code == 202

        # Check that defaults were applied
        call_args = mock_service.submit_batch.call_args[0][0]
        assert call_args[0].duration == 16.0  # Default
        assert call_args[0].temperature == 1.0  # Default

    def test_batch_handles_service_errors_gracefully(self, client, mock_service):
        """Test that batch handles service errors gracefully."""
        mock_service.submit_batch.side_effect = Exception("Service error")

        with patch("services.generation.api.service", mock_service):
            response = client.post(
                "/api/v1/generate/batch",
                json={
                    "requests": [
                        {"prompt": "prompt 1"},
                        {"prompt": "prompt 2"},
                    ]
                },
            )

        assert response.status_code == 500
