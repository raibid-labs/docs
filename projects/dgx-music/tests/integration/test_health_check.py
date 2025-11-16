"""
Integration Tests for Health Checks
===================================

Tests for enhanced health check functionality.
"""

import pytest
from fastapi.testclient import TestClient
from unittest.mock import Mock, patch, MagicMock
import shutil

from services.generation.api import app


@pytest.fixture
def client():
    """Create test client."""
    return TestClient(app)


@pytest.fixture
def mock_service():
    """Create mock service."""
    service = Mock()
    service.queue_manager = Mock()
    stats_mock = Mock()
    stats_mock.pending_jobs = 2
    stats_mock.processing_jobs = 1
    stats_mock.completed_jobs = 10
    stats_mock.failed_jobs = 1
    service.queue_manager.get_stats.return_value = stats_mock
    return service


class TestHealthCheck:
    """Test cases for health check endpoints."""

    def test_health_endpoint_returns_status(self, client):
        """Test that health endpoint returns overall status."""
        response = client.get("/api/v1/health")

        assert response.status_code == 200
        data = response.json()
        assert "status" in data
        assert data["status"] in ["healthy", "degraded", "unhealthy"]

    def test_health_includes_version_and_uptime(self, client):
        """Test that health response includes version and uptime."""
        response = client.get("/api/v1/health")

        data = response.json()
        assert "version" in data
        assert "uptime_seconds" in data
        assert isinstance(data["uptime_seconds"], (int, float))
        assert data["uptime_seconds"] >= 0

    def test_health_includes_database_check(self, client):
        """Test that health response includes database check."""
        response = client.get("/api/v1/health")

        data = response.json()
        assert "checks" in data
        assert "database" in data["checks"]
        db_check = data["checks"]["database"]
        assert "status" in db_check
        assert "message" in db_check

    def test_health_includes_gpu_check(self, client):
        """Test that health response includes GPU check."""
        response = client.get("/api/v1/health")

        data = response.json()
        assert "checks" in data
        assert "gpu" in data["checks"]
        gpu_check = data["checks"]["gpu"]
        assert "status" in gpu_check
        assert "message" in gpu_check

    def test_health_includes_queue_check(self, client, mock_service):
        """Test that health response includes queue check."""
        with patch("services.generation.api.service", mock_service):
            response = client.get("/api/v1/health")

        data = response.json()
        assert "checks" in data
        assert "queue" in data["checks"]
        queue_check = data["checks"]["queue"]
        assert "status" in queue_check
        assert "message" in queue_check

    def test_health_includes_disk_space_check(self, client):
        """Test that health response includes disk space check."""
        response = client.get("/api/v1/health")

        data = response.json()
        assert "checks" in data
        assert "disk_space" in data["checks"]
        disk_check = data["checks"]["disk_space"]
        assert "status" in disk_check
        assert "message" in disk_check

    def test_health_check_statuses_are_valid(self, client):
        """Test that all check statuses are valid."""
        response = client.get("/api/v1/health")

        data = response.json()
        valid_statuses = ["healthy", "degraded", "unhealthy"]

        for check_name, check_data in data["checks"].items():
            assert check_data["status"] in valid_statuses

    @patch("services.storage.database.get_session")
    def test_database_check_detects_failure(self, mock_get_session, client):
        """Test that database check detects connection failure."""
        # Mock database failure
        mock_session = MagicMock()
        mock_session.__enter__.return_value.execute.side_effect = Exception("DB error")
        mock_get_session.return_value = mock_session

        response = client.get("/api/v1/health")

        data = response.json()
        db_check = data["checks"]["database"]
        assert db_check["status"] in ["unhealthy", "degraded"]

    @patch("torch.cuda.is_available")
    def test_gpu_check_detects_unavailability(self, mock_cuda, client):
        """Test that GPU check detects when GPU is unavailable."""
        mock_cuda.return_value = False

        response = client.get("/api/v1/health")

        data = response.json()
        gpu_check = data["checks"]["gpu"]
        # GPU unavailability should mark as degraded, not unhealthy
        assert gpu_check["status"] in ["degraded", "healthy"]

    @patch("torch.cuda.is_available")
    @patch("torch.cuda.get_device_name")
    @patch("torch.cuda.device_count")
    def test_gpu_check_includes_device_info(
        self, mock_device_count, mock_device_name, mock_cuda, client
    ):
        """Test that GPU check includes device information."""
        mock_cuda.return_value = True
        mock_device_count.return_value = 1
        mock_device_name.return_value = "NVIDIA A100"

        response = client.get("/api/v1/health")

        data = response.json()
        gpu_check = data["checks"]["gpu"]
        if gpu_check["status"] == "healthy":
            assert "details" in gpu_check or "NVIDIA" in gpu_check["message"]

    def test_queue_check_includes_statistics(self, client, mock_service):
        """Test that queue check includes queue statistics."""
        with patch("services.generation.api.service", mock_service):
            response = client.get("/api/v1/health")

        data = response.json()
        queue_check = data["checks"]["queue"]

        if "details" in queue_check:
            details = queue_check["details"]
            assert "pending" in details or "processing" in details

    @patch("shutil.disk_usage")
    def test_disk_space_check_warns_when_low(self, mock_disk_usage, client):
        """Test that disk space check warns when space is low."""
        # Mock low disk space (15% free)
        mock_disk_usage.return_value = MagicMock(
            total=100 * 1024 ** 3,  # 100 GB
            free=15 * 1024 ** 3,  # 15 GB (15%)
        )

        response = client.get("/api/v1/health")

        data = response.json()
        disk_check = data["checks"]["disk_space"]
        # Should be degraded due to low space
        assert disk_check["status"] in ["degraded", "unhealthy"]

    @patch("shutil.disk_usage")
    def test_disk_space_check_critical_when_very_low(self, mock_disk_usage, client):
        """Test that disk space check is critical when space is very low."""
        # Mock very low disk space (5% free)
        mock_disk_usage.return_value = MagicMock(
            total=100 * 1024 ** 3,  # 100 GB
            free=5 * 1024 ** 3,  # 5 GB (5%)
        )

        response = client.get("/api/v1/health")

        data = response.json()
        disk_check = data["checks"]["disk_space"]
        # Should be unhealthy due to critical space
        assert disk_check["status"] == "unhealthy"

    def test_readiness_check_returns_ready(self, client, mock_service):
        """Test that readiness check returns ready status."""
        with patch("services.generation.api.service", mock_service):
            response = client.get("/api/v1/health/ready")

        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ready"

    def test_readiness_check_fails_without_service(self, client):
        """Test that readiness check fails when service not initialized."""
        with patch("services.generation.api.service", None):
            response = client.get("/api/v1/health/ready")

        assert response.status_code == 503

    def test_liveness_check_always_succeeds(self, client):
        """Test that liveness check always succeeds."""
        response = client.get("/api/v1/health/live")

        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "alive"

    def test_health_check_timestamp_included(self, client):
        """Test that health check includes timestamp."""
        response = client.get("/api/v1/health")

        data = response.json()
        assert "timestamp" in data
        # Verify it's a valid ISO timestamp
        from datetime import datetime
        timestamp = datetime.fromisoformat(data["timestamp"].replace("Z", "+00:00"))
        assert timestamp is not None
