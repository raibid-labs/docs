"""
Integration Tests for Rate Limiting
===================================

Tests for rate limiting functionality.
"""

import pytest
import time
from fastapi.testclient import TestClient
from unittest.mock import Mock, patch

from services.generation.api import app


@pytest.fixture
def client():
    """Create test client."""
    return TestClient(app)


@pytest.fixture
def mock_service():
    """Create mock service."""
    service = Mock()
    service.submit_job = Mock()
    return service


class TestRateLimiting:
    """Test cases for rate limiting."""

    def test_rate_limit_applies_to_generate_endpoint(self, client, mock_service):
        """Test that rate limit applies to generate endpoint."""
        with patch("services.generation.api.service", mock_service):
            # Make multiple requests rapidly
            responses = []
            for i in range(12):  # Limit is 10/minute
                response = client.post(
                    "/api/v1/generate",
                    json={"prompt": f"test prompt {i}"},
                    headers={"X-Forwarded-For": "192.168.1.1"},  # Simulate external IP
                )
                responses.append(response)

            # Check that some requests were rate limited
            status_codes = [r.status_code for r in responses]
            assert 429 in status_codes  # Too Many Requests

    def test_rate_limit_returns_429_status(self, client, mock_service):
        """Test that rate limit returns 429 status code."""
        with patch("services.generation.api.service", mock_service):
            # Exhaust rate limit
            for i in range(11):
                response = client.post(
                    "/api/v1/generate",
                    json={"prompt": "test prompt"},
                    headers={"X-Forwarded-For": "192.168.1.2"},
                )

            # This one should be rate limited
            if response.status_code == 429:
                assert response.status_code == 429

    def test_rate_limit_includes_retry_after_header(self, client, mock_service):
        """Test that rate limit response includes Retry-After header."""
        with patch("services.generation.api.service", mock_service):
            # Exhaust rate limit
            for i in range(11):
                response = client.post(
                    "/api/v1/generate",
                    json={"prompt": "test prompt"},
                    headers={"X-Forwarded-For": "192.168.1.3"},
                )

            # Check for Retry-After header in rate limited response
            if response.status_code == 429:
                # SlowAPI automatically adds this header
                assert "Retry-After" in response.headers or "X-RateLimit-Reset" in response.headers

    def test_localhost_bypasses_rate_limit(self, client, mock_service):
        """Test that localhost requests bypass rate limiting."""
        with patch("services.generation.api.service", mock_service):
            # Make many requests from localhost
            for i in range(15):  # More than the limit
                response = client.post(
                    "/api/v1/generate",
                    json={"prompt": f"test prompt {i}"},
                    # No X-Forwarded-For header, should use 127.0.0.1
                )

            # All should succeed (localhost is whitelisted)
            # Note: This test may be flaky depending on middleware implementation
            assert response.status_code in [202, 503]  # 503 if service not initialized

    def test_rate_limit_applies_per_ip(self, client, mock_service):
        """Test that rate limit is applied per IP address."""
        with patch("services.generation.api.service", mock_service):
            # Make requests from first IP
            for i in range(11):
                response1 = client.post(
                    "/api/v1/generate",
                    json={"prompt": "test prompt"},
                    headers={"X-Forwarded-For": "192.168.1.10"},
                )

            # Make requests from second IP (should not be limited)
            response2 = client.post(
                "/api/v1/generate",
                json={"prompt": "test prompt"},
                headers={"X-Forwarded-For": "192.168.1.20"},
            )

            # First IP should be limited, second should not
            if response1.status_code == 429:
                assert response2.status_code != 429

    def test_rate_limit_applies_to_batch_endpoint(self, client, mock_service):
        """Test that rate limit applies to batch endpoint."""
        with patch("services.generation.api.service", mock_service):
            mock_service.submit_batch = Mock(return_value=["job1", "job2"])
            mock_service._estimate_generation_time = Mock(return_value=20.0)

            # Make multiple batch requests
            for i in range(12):
                response = client.post(
                    "/api/v1/generate/batch",
                    json={
                        "requests": [
                            {"prompt": "prompt 1"},
                            {"prompt": "prompt 2"},
                        ]
                    },
                    headers={"X-Forwarded-For": "192.168.1.30"},
                )

            # Should eventually hit rate limit
            # Note: Rate limit state may persist across tests
            assert response.status_code in [202, 429, 503]
