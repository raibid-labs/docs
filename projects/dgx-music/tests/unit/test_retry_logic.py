"""
Unit Tests for Retry Logic
===========================

Tests for exponential backoff and retry behavior in the generation service.
"""

import pytest
import time
from unittest.mock import Mock, patch, MagicMock
from datetime import datetime

from services.generation.service import GenerationService
from services.generation.models import (
    GenerationRequest,
    GenerationStatus,
    GenerationStep,
    ModelName,
)
from services.generation.engine import GenerationError, ModelLoadError


class TestRetryLogic:
    """Test cases for retry logic."""

    @pytest.fixture
    def mock_engine(self):
        """Create a mock engine."""
        engine = Mock()
        engine.model_name = "small"
        engine.model = Mock()
        engine.generate_audio = Mock()
        engine.save_audio = Mock()
        engine.set_generation_params = Mock()
        return engine

    @pytest.fixture
    def mock_queue(self):
        """Create a mock queue manager."""
        queue = Mock()
        queue.enqueue = Mock()
        queue.dequeue = Mock()
        queue.complete_job = Mock()
        queue.fail_job = Mock()
        return queue

    @pytest.fixture
    def service(self, mock_engine, mock_queue):
        """Create service with mocked dependencies."""
        return GenerationService(
            engine=mock_engine,
            queue_manager=mock_queue,
            db_session_factory=None,
            max_retries=3,
            retry_delay=0.1,  # Short delay for testing
        )

    def test_initial_retry_count_is_zero(self, service):
        """Test that new jobs start with zero retries."""
        request = GenerationRequest(prompt="test prompt")
        response = service.submit_job(request)

        assert response.job_id.startswith("gen_")
        assert service._job_states[response.job_id]["retry_count"] == 0

    def test_successful_generation_no_retry(self, service, mock_engine, mock_queue):
        """Test that successful generation doesn't trigger retry."""
        # Setup
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        # Mock successful generation
        mock_engine.generate_audio.return_value = (Mock(), 32000)
        mock_engine.save_audio.return_value = Mock(
            duration=16.0,
            sample_rate=32000,
            channels=2,
            file_size_bytes=1000000,
            file_size_mb=1.0,
        )

        # Create job state
        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        # Mock queue to return job
        from services.generation.queue_manager import QueuedJob
        job = QueuedJob(job_id, request)
        mock_queue.dequeue.return_value = job

        # Process job
        result = service.process_next_job()

        assert result is True
        mock_queue.complete_job.assert_called_once_with(job_id)
        mock_queue.fail_job.assert_not_called()

    def test_failed_generation_triggers_retry(self, service, mock_engine, mock_queue):
        """Test that failed generation triggers retry."""
        # Setup
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        # Mock failed generation
        mock_engine.generate_audio.side_effect = GenerationError("Test error")

        # Create job state
        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        # Mock queue to return job
        from services.generation.queue_manager import QueuedJob
        job = QueuedJob(job_id, request, retry_count=0)
        mock_queue.dequeue.return_value = job

        # Process job
        result = service.process_next_job()

        assert result is True
        mock_queue.fail_job.assert_called_once_with(job_id)
        # Should re-enqueue with retry_count=1
        mock_queue.enqueue.assert_called_once()
        call_args = mock_queue.enqueue.call_args
        assert call_args[1]["retry_count"] == 1

    def test_exponential_backoff_delay(self, service):
        """Test that retry delay increases exponentially."""
        delays = []

        for retry_count in range(4):
            delay = service.retry_delay * (2 ** retry_count)
            delays.append(delay)

        assert delays[0] == 0.1  # Initial delay
        assert delays[1] == 0.2  # 2x
        assert delays[2] == 0.4  # 4x
        assert delays[3] == 0.8  # 8x

    @patch("time.sleep")
    def test_retry_waits_with_backoff(self, mock_sleep, service, mock_engine, mock_queue):
        """Test that retry waits with exponential backoff."""
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        # Simulate retry
        service._handle_retry(job_id, request, retry_count=1, error="Test error")

        # Check that sleep was called with correct delay
        expected_delay = service.retry_delay * (2 ** 1)
        mock_sleep.assert_called_once_with(expected_delay)

    def test_max_retries_reached_marks_failed(self, service, mock_engine, mock_queue):
        """Test that job is marked failed after max retries."""
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        # Mock failed generation
        mock_engine.generate_audio.side_effect = GenerationError("Test error")

        # Create job state with max retries already reached
        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 3,  # At max
        }

        # Mock queue to return job with max retries
        from services.generation.queue_manager import QueuedJob
        job = QueuedJob(job_id, request, retry_count=3)
        mock_queue.dequeue.return_value = job

        # Process job
        result = service.process_next_job()

        assert result is True
        mock_queue.fail_job.assert_called_once_with(job_id)
        # Should NOT re-enqueue
        mock_queue.enqueue.assert_not_called()
        # Should be marked as failed
        assert service._job_states[job_id]["status"] == GenerationStatus.FAILED

    def test_retry_count_increments_correctly(self, service, mock_queue):
        """Test that retry count increments with each retry."""
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        # Simulate multiple retries
        for expected_count in range(1, 4):
            service._handle_retry(
                job_id, request, retry_count=expected_count - 1, error="Test error"
            )

            # Check that state was updated
            assert service._job_states[job_id]["retry_count"] == expected_count

            # Check that re-enqueue was called with correct count
            call_args = mock_queue.enqueue.call_args
            assert call_args[1]["retry_count"] == expected_count

    def test_different_error_types_all_trigger_retry(self, service, mock_engine, mock_queue):
        """Test that different error types all trigger retry logic."""
        errors = [
            GenerationError("Generation failed"),
            ModelLoadError("Model load failed"),
            RuntimeError("Runtime error"),
            ValueError("Value error"),
        ]

        for error in errors:
            # Reset mocks
            mock_queue.reset_mock()

            request = GenerationRequest(prompt="test prompt")
            job_id = f"gen_test{errors.index(error)}"

            # Mock failed generation
            mock_engine.generate_audio.side_effect = error

            # Create job state
            service._job_states[job_id] = {
                "status": GenerationStatus.PENDING,
                "current_step": GenerationStep.QUEUED,
                "request": request,
                "created_at": datetime.utcnow(),
                "retry_count": 0,
            }

            # Mock queue to return job
            from services.generation.queue_manager import QueuedJob
            job = QueuedJob(job_id, request)
            mock_queue.dequeue.return_value = job

            # Process job
            service.process_next_job()

            # Should trigger retry for all error types
            mock_queue.enqueue.assert_called_once()

    def test_error_message_preserved_in_final_failure(self, service):
        """Test that error message is preserved when job finally fails."""
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"
        error_message = "Specific error message"

        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        # Handle final failure
        service._handle_failure(job_id, request, error_message, retry_count=3)

        # Check that error message is preserved
        assert error_message in service._job_states[job_id]["error_message"]
        assert "3 retries" in service._job_states[job_id]["error_message"]

    def test_retry_resets_status_to_pending(self, service, mock_queue):
        """Test that retry resets job status to pending."""
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        service._job_states[job_id] = {
            "status": GenerationStatus.PROCESSING,
            "current_step": GenerationStep.GENERATING,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        # Handle retry
        service._handle_retry(job_id, request, retry_count=0, error="Test error")

        # Check that status is reset
        assert service._job_states[job_id]["status"] == GenerationStatus.PENDING
        assert service._job_states[job_id]["current_step"] == GenerationStep.QUEUED

    def test_completed_at_set_on_final_failure(self, service):
        """Test that completed_at timestamp is set when job finally fails."""
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        before_time = datetime.utcnow()
        service._handle_failure(job_id, request, "Test error", retry_count=3)
        after_time = datetime.utcnow()

        completed_at = service._job_states[job_id]["completed_at"]
        assert completed_at is not None
        assert before_time <= completed_at <= after_time

    def test_configurable_max_retries(self):
        """Test that max_retries can be configured."""
        service1 = GenerationService(max_retries=2, retry_delay=0.1)
        service2 = GenerationService(max_retries=5, retry_delay=0.1)

        assert service1.max_retries == 2
        assert service2.max_retries == 5

    def test_configurable_retry_delay(self):
        """Test that retry_delay can be configured."""
        service1 = GenerationService(max_retries=3, retry_delay=0.5)
        service2 = GenerationService(max_retries=3, retry_delay=2.0)

        assert service1.retry_delay == 0.5
        assert service2.retry_delay == 2.0

    def test_retry_preserves_original_request(self, service, mock_queue):
        """Test that retry preserves the original request parameters."""
        request = GenerationRequest(
            prompt="specific prompt",
            duration=20.0,
            temperature=1.5,
            top_k=200,
        )
        job_id = "gen_test123"

        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        # Handle retry
        service._handle_retry(job_id, request, retry_count=0, error="Test error")

        # Check that the same request was re-enqueued
        call_args = mock_queue.enqueue.call_args
        enqueued_request = call_args[0][1]

        assert enqueued_request.prompt == request.prompt
        assert enqueued_request.duration == request.duration
        assert enqueued_request.temperature == request.temperature
        assert enqueued_request.top_k == request.top_k

    @patch("time.sleep")
    def test_multiple_retries_with_increasing_delays(self, mock_sleep, service, mock_queue):
        """Test that multiple retries use increasing delays."""
        request = GenerationRequest(prompt="test prompt")
        job_id = "gen_test123"

        service._job_states[job_id] = {
            "status": GenerationStatus.PENDING,
            "current_step": GenerationStep.QUEUED,
            "request": request,
            "created_at": datetime.utcnow(),
            "retry_count": 0,
        }

        # Simulate multiple retries
        for retry_count in range(3):
            service._handle_retry(job_id, request, retry_count=retry_count, error="Test error")

        # Check that sleep was called with increasing delays
        assert mock_sleep.call_count == 3
        calls = mock_sleep.call_args_list

        # Delays should be 0.1, 0.2, 0.4 (exponential backoff)
        assert calls[0][0][0] == pytest.approx(0.1)
        assert calls[1][0][0] == pytest.approx(0.2)
        assert calls[2][0][0] == pytest.approx(0.4)
