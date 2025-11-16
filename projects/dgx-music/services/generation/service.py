"""
Music Generation Service with Retry Logic and Progress Tracking
===============================================================

High-level service layer that orchestrates the generation pipeline:
- Job queue management
- Retry logic with exponential backoff
- Progress tracking through pipeline steps
- Error handling and recovery
- Database integration
"""

import time
import traceback
from datetime import datetime
from pathlib import Path
from typing import Optional, Dict, Any
from uuid import uuid4

from .engine import MusicGenerationEngine, GenerationError, ModelLoadError
from .models import (
    GenerationRequest,
    GenerationResponse,
    GenerationResult,
    GenerationStatus,
    GenerationStep,
    AudioMetadata,
)
from .queue_manager import QueueManager, get_queue_manager
from .config import settings
from .logger import get_logger, LogContext

logger = get_logger("service")


class GenerationService:
    """
    High-level generation service with retry logic and progress tracking.

    This service handles:
    - Job submission and queue management
    - Retry logic with exponential backoff
    - Progress tracking through pipeline steps
    - Database integration for persistence
    - Error handling and recovery
    """

    def __init__(
        self,
        engine: Optional[MusicGenerationEngine] = None,
        queue_manager: Optional[QueueManager] = None,
        db_session_factory: Optional[Any] = None,
        max_retries: int = 3,
        retry_delay: float = 1.0,
    ):
        """
        Initialize generation service.

        Args:
            engine: Music generation engine (created if not provided)
            queue_manager: Queue manager (created if not provided)
            db_session_factory: Factory to create database sessions
            max_retries: Maximum retry attempts per job
            retry_delay: Initial retry delay in seconds (exponential backoff)
        """
        self.engine = engine or MusicGenerationEngine(
            model_name=settings.model_name.replace("musicgen-", ""),
            use_gpu=settings.use_gpu,
            enable_caching=settings.enable_model_caching,
        )
        self.queue_manager = queue_manager or get_queue_manager(db_session_factory)
        self.db_session_factory = db_session_factory
        self.max_retries = max_retries
        self.retry_delay = retry_delay

        # Job state tracking
        self._job_states: Dict[str, Dict[str, Any]] = {}

        logger.info(
            f"Generation service initialized: "
            f"max_retries={max_retries}, retry_delay={retry_delay}s"
        )

    def submit_job(self, request: GenerationRequest) -> GenerationResponse:
        """
        Submit a generation job to the queue.

        Args:
            request: Generation request

        Returns:
            Generation response with job_id
        """
        job_id = f"gen_{uuid4().hex[:8]}"

        with LogContext(job_id=job_id):
            logger.info(f"Submitting job: prompt='{request.prompt[:50]}...'")

            # Create job state
            self._job_states[job_id] = {
                "status": GenerationStatus.PENDING,
                "current_step": GenerationStep.QUEUED,
                "request": request,
                "created_at": datetime.utcnow(),
                "retry_count": 0,
            }

            # Add to database if available
            if self.db_session_factory:
                self._create_database_record(job_id, request)

            # Enqueue job
            self.queue_manager.enqueue(job_id, request)

            # Calculate estimated time
            estimated_time = self._estimate_generation_time(request)

            return GenerationResponse(
                job_id=job_id,
                status=GenerationStatus.PENDING,
                current_step=GenerationStep.QUEUED,
                estimated_time_seconds=estimated_time,
                created_at=datetime.utcnow(),
            )

    def submit_batch(self, requests: list[GenerationRequest]) -> list[str]:
        """
        Submit multiple jobs atomically.

        Args:
            requests: List of generation requests

        Returns:
            List of job IDs
        """
        job_ids = []
        jobs = []

        for request in requests:
            job_id = f"gen_{uuid4().hex[:8]}"
            job_ids.append(job_id)
            jobs.append((job_id, request))

            # Create job state
            self._job_states[job_id] = {
                "status": GenerationStatus.PENDING,
                "current_step": GenerationStep.QUEUED,
                "request": request,
                "created_at": datetime.utcnow(),
                "retry_count": 0,
            }

            # Add to database if available
            if self.db_session_factory:
                self._create_database_record(job_id, request)

        # Enqueue all jobs atomically
        self.queue_manager.enqueue_batch(jobs)

        logger.info(f"Batch submitted: {len(job_ids)} jobs")

        return job_ids

    def get_job_status(self, job_id: str) -> Optional[GenerationResult]:
        """
        Get the status of a job.

        Args:
            job_id: Job identifier

        Returns:
            Generation result or None if not found
        """
        # Check in-memory state first
        if job_id in self._job_states:
            state = self._job_states[job_id]
            return self._state_to_result(job_id, state)

        # Check database if available
        if self.db_session_factory:
            return self._get_from_database(job_id)

        return None

    def cancel_job(self, job_id: str) -> bool:
        """
        Cancel a pending job.

        Args:
            job_id: Job identifier

        Returns:
            True if cancelled, False if not found or already processing
        """
        # Try to cancel in queue
        if self.queue_manager.cancel_job(job_id):
            # Update in-memory state
            if job_id in self._job_states:
                self._job_states[job_id]["status"] = GenerationStatus.CANCELLED
                self._job_states[job_id]["current_step"] = GenerationStep.FAILED

            # Update database if available
            if self.db_session_factory:
                self._update_database_status(
                    job_id, GenerationStatus.CANCELLED, error_message="Cancelled by user"
                )

            logger.info(f"Job cancelled: {job_id}")
            return True

        return False

    def process_next_job(self) -> bool:
        """
        Process the next job in the queue.

        Returns:
            True if a job was processed, False if queue is empty
        """
        # Get next job from queue
        job = self.queue_manager.dequeue()
        if not job:
            return False

        job_id = job.job_id
        request = job.request
        retry_count = job.retry_count

        with LogContext(job_id=job_id):
            logger.info(f"Processing job: {job_id} (retry_count={retry_count})")

            try:
                # Generate with progress tracking
                result = self._generate_with_progress(job_id, request, retry_count)

                # Mark as complete
                self.queue_manager.complete_job(job_id)

                # Update state
                if job_id in self._job_states:
                    self._job_states[job_id].update({
                        "status": result.status,
                        "current_step": result.current_step,
                        "completed_at": result.completed_at,
                    })

                return True

            except Exception as e:
                logger.error(f"Job processing failed: {e}")
                logger.error(traceback.format_exc())

                # Handle retry logic
                if retry_count < self.max_retries:
                    self._handle_retry(job_id, request, retry_count, str(e))
                else:
                    self._handle_failure(job_id, request, str(e), retry_count)

                self.queue_manager.fail_job(job_id)
                return True

    def _generate_with_progress(
        self, job_id: str, request: GenerationRequest, retry_count: int
    ) -> GenerationResult:
        """
        Generate audio with progress tracking.

        Args:
            job_id: Job identifier
            request: Generation request
            retry_count: Current retry count

        Returns:
            Generation result

        Raises:
            GenerationError: If generation fails
        """
        start_time = time.time()

        # Update status: loading model
        self._update_progress(job_id, GenerationStatus.PROCESSING, GenerationStep.LOADING_MODEL)

        try:
            # Ensure model is loaded
            if self.engine.model is None:
                self.engine.load_model()

            # Update status: encoding prompt
            self._update_progress(job_id, GenerationStatus.PROCESSING, GenerationStep.ENCODING_PROMPT)

            # Set generation parameters
            self.engine.set_generation_params(
                duration=request.duration,
                temperature=request.temperature,
                top_k=request.top_k,
                top_p=request.top_p,
                cfg_coef=request.cfg_coef,
            )

            # Update status: generating
            self._update_progress(job_id, GenerationStatus.PROCESSING, GenerationStep.GENERATING)

            # Generate audio
            audio, sample_rate = self.engine.generate_audio(
                prompt=request.prompt,
                duration=request.duration,
                temperature=request.temperature,
                top_k=request.top_k,
                top_p=request.top_p,
                cfg_coef=request.cfg_coef,
            )

            # Update status: saving
            self._update_progress(job_id, GenerationStatus.PROCESSING, GenerationStep.SAVING)

            # Save to file
            output_path = settings.get_output_path(job_id)
            metadata = self.engine.save_audio(
                audio=audio,
                sample_rate=sample_rate,
                output_path=output_path,
                normalize=settings.normalize_audio,
            )

            # Calculate total time
            total_time = time.time() - start_time

            # Update status: completed
            self._update_progress(job_id, GenerationStatus.COMPLETED, GenerationStep.COMPLETED)

            # Build result
            result = GenerationResult(
                job_id=job_id,
                status=GenerationStatus.COMPLETED,
                current_step=GenerationStep.COMPLETED,
                prompt=request.prompt,
                model=f"musicgen-{self.engine.model_name}",
                file_url=f"/api/v1/files/{output_path.name}",
                file_path=str(output_path),
                metadata=metadata,
                generation_time_seconds=total_time,
                retry_count=retry_count,
                created_at=self._job_states.get(job_id, {}).get("created_at", datetime.utcnow()),
                completed_at=datetime.utcnow(),
            )

            # Update database
            if self.db_session_factory:
                self._complete_database_record(job_id, result)

            logger.info(f"Job completed: {job_id} in {total_time:.2f}s")

            return result

        except Exception as e:
            logger.error(f"Generation failed: {e}")
            raise GenerationError(f"Generation failed: {e}")

    def _update_progress(
        self, job_id: str, status: GenerationStatus, step: GenerationStep
    ) -> None:
        """
        Update job progress.

        Args:
            job_id: Job identifier
            status: Current status
            step: Current pipeline step
        """
        # Update in-memory state
        if job_id in self._job_states:
            self._job_states[job_id]["status"] = status
            self._job_states[job_id]["current_step"] = step

        # Update database if available
        if self.db_session_factory:
            self._update_database_progress(job_id, status, step)

        logger.debug(f"Job progress: {job_id} -> {status.value}/{step.value}")

    def _handle_retry(
        self, job_id: str, request: GenerationRequest, retry_count: int, error: str
    ) -> None:
        """
        Handle job retry with exponential backoff.

        Args:
            job_id: Job identifier
            request: Generation request
            retry_count: Current retry count
            error: Error message
        """
        # Calculate backoff delay
        delay = self.retry_delay * (2 ** retry_count)

        logger.warning(
            f"Job failed, will retry: {job_id} "
            f"(attempt {retry_count + 1}/{self.max_retries}, delay={delay:.1f}s)"
        )

        # Wait before retrying
        time.sleep(delay)

        # Re-enqueue with incremented retry count
        self.queue_manager.enqueue(job_id, request, retry_count=retry_count + 1)

        # Update state
        if job_id in self._job_states:
            self._job_states[job_id]["retry_count"] = retry_count + 1
            self._job_states[job_id]["status"] = GenerationStatus.PENDING
            self._job_states[job_id]["current_step"] = GenerationStep.QUEUED

    def _handle_failure(
        self, job_id: str, request: GenerationRequest, error: str, retry_count: int
    ) -> None:
        """
        Handle final job failure after all retries.

        Args:
            job_id: Job identifier
            request: Generation request
            error: Error message
            retry_count: Final retry count
        """
        logger.error(
            f"Job failed after {retry_count} retries: {job_id} - {error}"
        )

        # Update in-memory state
        if job_id in self._job_states:
            self._job_states[job_id].update({
                "status": GenerationStatus.FAILED,
                "current_step": GenerationStep.FAILED,
                "error_message": error,
                "retry_count": retry_count,
                "completed_at": datetime.utcnow(),
            })

        # Update database
        if self.db_session_factory:
            self._update_database_status(
                job_id,
                GenerationStatus.FAILED,
                error_message=f"Failed after {retry_count} retries: {error}",
            )

    def _estimate_generation_time(self, request: GenerationRequest) -> float:
        """
        Estimate generation time for a request.

        Args:
            request: Generation request

        Returns:
            Estimated time in seconds
        """
        # Base estimate: ~1.2x real-time on GPU, ~10x on CPU
        rtf = 1.2 if settings.use_gpu else 10.0
        return request.duration * rtf

    def _state_to_result(self, job_id: str, state: Dict[str, Any]) -> GenerationResult:
        """
        Convert internal state to GenerationResult.

        Args:
            job_id: Job identifier
            state: Internal state dictionary

        Returns:
            Generation result
        """
        request = state["request"]
        return GenerationResult(
            job_id=job_id,
            status=state["status"],
            current_step=state["current_step"],
            prompt=request.prompt,
            model=request.model.value,
            retry_count=state.get("retry_count", 0),
            error_message=state.get("error_message"),
            created_at=state["created_at"],
            completed_at=state.get("completed_at"),
        )

    def _create_database_record(self, job_id: str, request: GenerationRequest) -> None:
        """Create database record for job."""
        try:
            from services.storage.database import create_generation

            with self.db_session_factory() as session:
                create_generation(
                    session=session,
                    prompt=request.prompt,
                    model_name=request.model.value,
                    duration_seconds=request.duration,
                    sample_rate=settings.sample_rate,
                    channels=settings.channels,
                    file_path=str(settings.get_output_path(job_id)),
                )
                session.commit()

        except Exception as e:
            logger.error(f"Failed to create database record: {e}")

    def _update_database_status(
        self, job_id: str, status: GenerationStatus, error_message: Optional[str] = None
    ) -> None:
        """Update job status in database."""
        try:
            from services.storage.database import update_generation_status

            with self.db_session_factory() as session:
                update_generation_status(
                    session=session,
                    generation_id=job_id,
                    status=status.value,
                    error_message=error_message,
                )
                session.commit()

        except Exception as e:
            logger.error(f"Failed to update database status: {e}")

    def _update_database_progress(
        self, job_id: str, status: GenerationStatus, step: GenerationStep
    ) -> None:
        """Update job progress in database."""
        # For now, just update status
        # In a real implementation, you would store the step in a separate field
        self._update_database_status(job_id, status)

    def _complete_database_record(self, job_id: str, result: GenerationResult) -> None:
        """Complete database record with final result."""
        try:
            from services.storage.database import complete_generation

            with self.db_session_factory() as session:
                complete_generation(
                    session=session,
                    generation_id=job_id,
                    generation_time=result.generation_time_seconds or 0.0,
                    file_size_bytes=result.metadata.file_size_bytes if result.metadata else 0,
                )
                session.commit()

        except Exception as e:
            logger.error(f"Failed to complete database record: {e}")

    def _get_from_database(self, job_id: str) -> Optional[GenerationResult]:
        """Get job from database."""
        try:
            from services.storage.database import get_generation

            with self.db_session_factory() as session:
                gen = get_generation(session, job_id)
                if not gen:
                    return None

                # Convert to GenerationResult
                return GenerationResult(
                    job_id=gen.id,
                    status=GenerationStatus(gen.status),
                    current_step=GenerationStep.COMPLETED if gen.is_complete else GenerationStep.QUEUED,
                    prompt=gen.prompt,
                    model=gen.model_name,
                    file_path=gen.file_path,
                    file_url=f"/api/v1/files/{Path(gen.file_path).name}",
                    generation_time_seconds=gen.generation_time_seconds,
                    error_message=gen.error_message,
                    created_at=gen.created_at,
                    completed_at=gen.completed_at,
                )

        except Exception as e:
            logger.error(f"Failed to get from database: {e}")
            return None


# Global service instance
_service: Optional[GenerationService] = None


def get_service(db_session_factory: Optional[Any] = None) -> GenerationService:
    """
    Get or create the global generation service instance.

    Args:
        db_session_factory: Factory to create database sessions

    Returns:
        GenerationService instance
    """
    global _service
    if _service is None:
        _service = GenerationService(
            db_session_factory=db_session_factory,
            max_retries=getattr(settings, "max_retries", 3),
            retry_delay=getattr(settings, "retry_delay_seconds", 1.0),
        )
    return _service
