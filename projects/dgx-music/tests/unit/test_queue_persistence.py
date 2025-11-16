"""
Unit Tests for Queue Persistence
=================================

Tests for queue state persistence and recovery functionality.
"""

import pytest
from unittest.mock import Mock, MagicMock, patch
from datetime import datetime
from collections import deque

from services.generation.queue_manager import QueueManager, QueuedJob
from services.generation.models import GenerationRequest, ModelName


class TestQueuePersistence:
    """Test cases for queue persistence."""

    @pytest.fixture
    def queue_manager(self):
        """Create a queue manager without database."""
        return QueueManager(db_session_factory=None)

    @pytest.fixture
    def mock_db_session(self):
        """Create a mock database session."""
        session = Mock()
        session.commit = Mock()
        session.query = Mock()
        session.__enter__ = Mock(return_value=session)
        session.__exit__ = Mock(return_value=False)
        return session

    @pytest.fixture
    def db_session_factory(self, mock_db_session):
        """Create a mock session factory."""
        return Mock(return_value=mock_db_session)

    def test_queue_manager_initializes_empty(self, queue_manager):
        """Test that queue manager starts with empty queue."""
        assert queue_manager.get_queue_length() == 0
        assert queue_manager.is_empty() is True

    def test_enqueue_adds_job(self, queue_manager):
        """Test that enqueue adds job to queue."""
        request = GenerationRequest(prompt="test prompt")
        queue_manager.enqueue("job1", request)

        assert queue_manager.get_queue_length() == 1
        assert queue_manager.is_empty() is False

    def test_dequeue_returns_fifo_order(self, queue_manager):
        """Test that dequeue returns jobs in FIFO order."""
        request1 = GenerationRequest(prompt="prompt 1")
        request2 = GenerationRequest(prompt="prompt 2")
        request3 = GenerationRequest(prompt="prompt 3")

        queue_manager.enqueue("job1", request1)
        queue_manager.enqueue("job2", request2)
        queue_manager.enqueue("job3", request3)

        job1 = queue_manager.dequeue()
        job2 = queue_manager.dequeue()
        job3 = queue_manager.dequeue()

        assert job1.job_id == "job1"
        assert job2.job_id == "job2"
        assert job3.job_id == "job3"

    def test_enqueue_batch_adds_all_jobs(self, queue_manager):
        """Test that enqueue_batch adds all jobs atomically."""
        requests = [
            ("job1", GenerationRequest(prompt="prompt 1")),
            ("job2", GenerationRequest(prompt="prompt 2")),
            ("job3", GenerationRequest(prompt="prompt 3")),
        ]

        queue_manager.enqueue_batch(requests)

        assert queue_manager.get_queue_length() == 3

    def test_complete_job_clears_current_job(self, queue_manager):
        """Test that complete_job clears current job."""
        request = GenerationRequest(prompt="test prompt")
        queue_manager.enqueue("job1", request)

        job = queue_manager.dequeue()
        assert queue_manager._current_job is not None

        queue_manager.complete_job("job1")
        assert queue_manager._current_job is None

    def test_fail_job_clears_current_job(self, queue_manager):
        """Test that fail_job clears current job."""
        request = GenerationRequest(prompt="test prompt")
        queue_manager.enqueue("job1", request)

        job = queue_manager.dequeue()
        assert queue_manager._current_job is not None

        queue_manager.fail_job("job1")
        assert queue_manager._current_job is None

    def test_cancel_pending_job_succeeds(self, queue_manager):
        """Test that cancelling a pending job succeeds."""
        request = GenerationRequest(prompt="test prompt")
        queue_manager.enqueue("job1", request)

        success = queue_manager.cancel_job("job1")

        assert success is True
        assert queue_manager.get_queue_length() == 0

    def test_cancel_current_job_fails(self, queue_manager):
        """Test that cancelling a currently processing job fails."""
        request = GenerationRequest(prompt="test prompt")
        queue_manager.enqueue("job1", request)

        job = queue_manager.dequeue()
        success = queue_manager.cancel_job("job1")

        assert success is False

    def test_cancel_nonexistent_job_fails(self, queue_manager):
        """Test that cancelling a non-existent job fails."""
        success = queue_manager.cancel_job("nonexistent")
        assert success is False

    def test_get_position_returns_correct_index(self, queue_manager):
        """Test that get_position returns correct queue position."""
        for i in range(5):
            queue_manager.enqueue(f"job{i}", GenerationRequest(prompt=f"prompt {i}"))

        assert queue_manager.get_position("job0") == 1  # First in queue (0 is current)
        assert queue_manager.get_position("job2") == 3
        assert queue_manager.get_position("job4") == 5

    def test_get_position_for_current_job(self, queue_manager):
        """Test that get_position returns 0 for current job."""
        queue_manager.enqueue("job1", GenerationRequest(prompt="prompt 1"))
        queue_manager.enqueue("job2", GenerationRequest(prompt="prompt 2"))

        job = queue_manager.dequeue()
        assert queue_manager.get_position("job1") == 0

    def test_get_stats_returns_correct_counts(self, queue_manager):
        """Test that get_stats returns correct statistics."""
        # Add some jobs
        for i in range(3):
            queue_manager.enqueue(f"job{i}", GenerationRequest(prompt=f"prompt {i}"))

        # Process one
        job = queue_manager.dequeue()
        queue_manager.complete_job(job.job_id)

        stats = queue_manager.get_stats()

        assert stats.pending_jobs == 2
        assert stats.processing_jobs == 0
        assert stats.completed_jobs == 1

    def test_clear_removes_all_jobs(self, queue_manager):
        """Test that clear removes all pending jobs."""
        for i in range(5):
            queue_manager.enqueue(f"job{i}", GenerationRequest(prompt=f"prompt {i}"))

        count = queue_manager.clear()

        assert count == 5
        assert queue_manager.get_queue_length() == 0

    def test_queued_job_preserves_retry_count(self, queue_manager):
        """Test that QueuedJob preserves retry count."""
        request = GenerationRequest(prompt="test prompt")
        queue_manager.enqueue("job1", request, retry_count=2)

        job = queue_manager.dequeue()
        assert job.retry_count == 2

    def test_queued_job_to_dict_serialization(self):
        """Test that QueuedJob can be serialized to dict."""
        request = GenerationRequest(prompt="test prompt", duration=20.0)
        job = QueuedJob("job1", request, retry_count=1)

        data = job.to_dict()

        assert data["job_id"] == "job1"
        assert data["request"]["prompt"] == "test prompt"
        assert data["request"]["duration"] == 20.0
        assert data["retry_count"] == 1
        assert "created_at" in data
        assert "enqueued_at" in data

    def test_queued_job_from_dict_deserialization(self):
        """Test that QueuedJob can be deserialized from dict."""
        data = {
            "job_id": "job1",
            "request": {
                "prompt": "test prompt",
                "duration": 20.0,
                "temperature": 1.0,
                "top_k": 250,
                "top_p": 0.0,
                "cfg_coef": 3.0,
                "model": "musicgen-small",
            },
            "created_at": "2025-11-07T10:00:00",
            "retry_count": 2,
            "enqueued_at": "2025-11-07T10:00:00",
        }

        job = QueuedJob.from_dict(data)

        assert job.job_id == "job1"
        assert job.request.prompt == "test prompt"
        assert job.request.duration == 20.0
        assert job.retry_count == 2

    def test_shutdown_logs_current_state(self, queue_manager, caplog):
        """Test that shutdown logs current queue state."""
        for i in range(3):
            queue_manager.enqueue(f"job{i}", GenerationRequest(prompt=f"prompt {i}"))

        queue_manager.shutdown()

        # Check that shutdown was logged
        assert "Queue manager shutting down" in caplog.text
        assert "pending=3" in caplog.text

    @patch("atexit.register")
    def test_atexit_handler_registered(self, mock_atexit):
        """Test that shutdown handler is registered with atexit."""
        queue = QueueManager()
        mock_atexit.assert_called_once()

    def test_stats_track_completed_jobs(self, queue_manager):
        """Test that stats correctly track completed jobs."""
        queue_manager.enqueue("job1", GenerationRequest(prompt="prompt 1"))
        job = queue_manager.dequeue()
        queue_manager.complete_job("job1")

        stats = queue_manager.get_stats()
        assert stats.completed_jobs == 1

    def test_stats_track_failed_jobs(self, queue_manager):
        """Test that stats correctly track failed jobs."""
        queue_manager.enqueue("job1", GenerationRequest(prompt="prompt 1"))
        job = queue_manager.dequeue()
        queue_manager.fail_job("job1")

        stats = queue_manager.get_stats()
        assert stats.failed_jobs == 1

    def test_stats_track_cancelled_jobs(self, queue_manager):
        """Test that stats correctly track cancelled jobs."""
        queue_manager.enqueue("job1", GenerationRequest(prompt="prompt 1"))
        queue_manager.cancel_job("job1")

        # Note: cancelled count is tracked in _stats but not exposed in QueueStats
        # This is by design - cancelled jobs are just removed from queue
        assert queue_manager._stats["total_cancelled"] == 1

    def test_oldest_job_age_calculated_correctly(self, queue_manager):
        """Test that oldest job age is calculated correctly."""
        import time

        queue_manager.enqueue("job1", GenerationRequest(prompt="prompt 1"))
        time.sleep(0.1)  # Small delay

        stats = queue_manager.get_stats()

        assert stats.oldest_pending_job_age_seconds is not None
        assert stats.oldest_pending_job_age_seconds >= 0.1
