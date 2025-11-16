"""
REST API for Music Generation Service
====================================

FastAPI application providing:
- Generation endpoints (single and batch)
- Job status and cancellation
- Rate limiting
- Enhanced health checks
- OpenAPI documentation
"""

import time
import shutil
from datetime import datetime
from pathlib import Path
from typing import Optional

from fastapi import FastAPI, HTTPException, Request, Response
from fastapi.responses import JSONResponse, FileResponse
from fastapi.middleware.cors import CORSMiddleware
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded

from .models import (
    GenerationRequest,
    GenerationResponse,
    GenerationResult,
    GenerationStatus,
    BatchGenerationRequest,
    BatchGenerationResponse,
    HealthStatus,
    HealthCheck,
    QueueStats,
)
from .service import get_service, GenerationService
from .config import settings
from .logger import get_logger

logger = get_logger("api")

# Initialize rate limiter
limiter = Limiter(key_func=get_remote_address)

# Create FastAPI app
app = FastAPI(
    title="DGX Music Generation API",
    description="AI-powered music generation service using MusicGen",
    version=settings.version,
    docs_url="/api/v1/docs",
    redoc_url="/api/v1/redoc",
    openapi_url="/api/v1/openapi.json",
)

# Add rate limiter to app
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Track startup time for uptime calculation
startup_time = time.time()

# Initialize service (will be set up on startup)
service: Optional[GenerationService] = None


@app.on_event("startup")
async def startup_event():
    """Initialize service on startup."""
    global service

    logger.info("Starting DGX Music Generation API")

    # Initialize database
    try:
        from services.storage.database import init_db, get_session

        init_db()
        logger.info("Database initialized")

        # Initialize service with database session factory
        service = get_service(db_session_factory=get_session)
        logger.info("Generation service initialized")

    except Exception as e:
        logger.error(f"Failed to initialize service: {e}")
        # Continue without database for testing
        service = get_service()


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown."""
    logger.info("Shutting down DGX Music Generation API")


# ============================================================================
# Generation Endpoints
# ============================================================================


@app.post(
    "/api/v1/generate",
    response_model=GenerationResponse,
    status_code=202,
    summary="Generate music from text prompt",
    description="""
    Submit a music generation request. The job will be queued and processed asynchronously.

    Rate limit: 10 requests per minute per IP address.

    Returns a job ID that can be used to check status and retrieve the generated audio.
    """,
    tags=["Generation"],
)
@limiter.limit("10/minute")
async def generate_music(request: Request, generation_request: GenerationRequest) -> GenerationResponse:
    """
    Generate music from a text prompt.

    Args:
        generation_request: Generation parameters

    Returns:
        Job information with ID for status checking

    Raises:
        HTTPException: If service is not initialized or request is invalid
    """
    if not service:
        raise HTTPException(status_code=503, detail="Service not initialized")

    try:
        response = service.submit_job(generation_request)
        logger.info(f"Job submitted: {response.job_id}")
        return response

    except Exception as e:
        logger.error(f"Failed to submit job: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post(
    "/api/v1/generate/batch",
    response_model=BatchGenerationResponse,
    status_code=202,
    summary="Generate multiple music tracks in batch",
    description="""
    Submit multiple generation requests in a single batch.
    All jobs will be queued atomically.

    Rate limit: 10 requests per minute per IP address.

    Maximum 10 requests per batch.
    """,
    tags=["Generation"],
)
@limiter.limit("10/minute")
async def generate_batch(
    request: Request, batch_request: BatchGenerationRequest
) -> BatchGenerationResponse:
    """
    Generate multiple music tracks in batch.

    Args:
        batch_request: Batch of generation requests

    Returns:
        List of job IDs

    Raises:
        HTTPException: If service is not initialized or request is invalid
    """
    if not service:
        raise HTTPException(status_code=503, detail="Service not initialized")

    try:
        job_ids = service.submit_batch(batch_request.requests)

        # Calculate total estimated time
        total_time = sum(
            service._estimate_generation_time(req) for req in batch_request.requests
        )

        logger.info(f"Batch submitted: {len(job_ids)} jobs")

        return BatchGenerationResponse(
            job_ids=job_ids,
            total_jobs=len(job_ids),
            estimated_total_time_seconds=total_time,
        )

    except Exception as e:
        logger.error(f"Failed to submit batch: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================================
# Job Management Endpoints
# ============================================================================


@app.get(
    "/api/v1/jobs/{job_id}",
    response_model=GenerationResult,
    summary="Get job status and result",
    description="""
    Get the status and result of a generation job.

    Returns detailed information including:
    - Current status (pending, processing, completed, failed)
    - Current pipeline step (queued, loading_model, encoding_prompt, generating, saving)
    - Progress information
    - Generated file URL (if completed)
    - Error message (if failed)
    """,
    tags=["Jobs"],
)
async def get_job_status(job_id: str) -> GenerationResult:
    """
    Get the status of a generation job.

    Args:
        job_id: Job identifier

    Returns:
        Job status and result

    Raises:
        HTTPException: If job not found
    """
    if not service:
        raise HTTPException(status_code=503, detail="Service not initialized")

    result = service.get_job_status(job_id)
    if not result:
        raise HTTPException(status_code=404, detail="Job not found")

    return result


@app.delete(
    "/api/v1/jobs/{job_id}",
    status_code=200,
    summary="Cancel a pending job",
    description="""
    Cancel a pending job in the queue.

    **Note**: Only pending jobs can be cancelled. Jobs that are currently
    processing cannot be cancelled and will return an error.

    Returns:
        - 200: Job successfully cancelled
        - 404: Job not found
        - 409: Job is currently processing and cannot be cancelled
    """,
    tags=["Jobs"],
)
async def cancel_job(job_id: str) -> dict:
    """
    Cancel a pending generation job.

    Args:
        job_id: Job identifier

    Returns:
        Success message

    Raises:
        HTTPException: If job not found or cannot be cancelled
    """
    if not service:
        raise HTTPException(status_code=503, detail="Service not initialized")

    # Check if job exists
    result = service.get_job_status(job_id)
    if not result:
        raise HTTPException(status_code=404, detail="Job not found")

    # Check if job is already completed or failed
    if result.status in [GenerationStatus.COMPLETED, GenerationStatus.FAILED]:
        raise HTTPException(
            status_code=409,
            detail=f"Cannot cancel job with status: {result.status.value}",
        )

    # Check if job is currently processing
    if result.status == GenerationStatus.PROCESSING:
        raise HTTPException(
            status_code=409,
            detail="Cannot cancel job that is currently processing",
        )

    # Try to cancel
    success = service.cancel_job(job_id)
    if not success:
        raise HTTPException(
            status_code=409,
            detail="Job is currently processing and cannot be cancelled",
        )

    logger.info(f"Job cancelled: {job_id}")

    return {
        "message": "Job cancelled successfully",
        "job_id": job_id,
        "cancelled_at": datetime.utcnow().isoformat(),
    }


@app.get(
    "/api/v1/files/{filename}",
    response_class=FileResponse,
    summary="Download generated audio file",
    description="Download a generated WAV file by filename.",
    tags=["Files"],
)
async def download_file(filename: str):
    """
    Download a generated audio file.

    Args:
        filename: Name of the file to download

    Returns:
        Audio file

    Raises:
        HTTPException: If file not found
    """
    file_path = settings.output_dir / filename

    if not file_path.exists():
        raise HTTPException(status_code=404, detail="File not found")

    return FileResponse(
        path=str(file_path),
        media_type="audio/wav",
        filename=filename,
    )


# ============================================================================
# Queue Management
# ============================================================================


@app.get(
    "/api/v1/queue/stats",
    response_model=QueueStats,
    summary="Get queue statistics",
    description="Get statistics about the job queue including pending, processing, and completed jobs.",
    tags=["Queue"],
)
async def get_queue_stats() -> QueueStats:
    """
    Get queue statistics.

    Returns:
        Queue statistics
    """
    if not service:
        raise HTTPException(status_code=503, detail="Service not initialized")

    return service.queue_manager.get_stats()


# ============================================================================
# Health Check
# ============================================================================


@app.get(
    "/api/v1/health",
    response_model=HealthStatus,
    summary="Enhanced health check",
    description="""
    Comprehensive health check that verifies:
    - Database connectivity
    - GPU availability (if configured)
    - Queue status
    - Disk space in output directory
    - Overall service health

    Returns one of:
    - healthy: All checks passed
    - degraded: Some non-critical checks failed
    - unhealthy: Critical checks failed
    """,
    tags=["Health"],
)
async def health_check() -> HealthStatus:
    """
    Comprehensive health check.

    Returns:
        Health status with detailed checks
    """
    checks = {}
    overall_status = "healthy"

    # Check database connectivity
    try:
        from services.storage.database import get_session

        with get_session() as session:
            # Try a simple query
            session.execute("SELECT 1")

        checks["database"] = HealthCheck(
            name="database",
            status="healthy",
            message="Connected",
        )
    except Exception as e:
        checks["database"] = HealthCheck(
            name="database",
            status="unhealthy",
            message=f"Failed: {str(e)}",
        )
        overall_status = "unhealthy"

    # Check GPU availability
    try:
        import torch

        if torch.cuda.is_available():
            device_count = torch.cuda.device_count()
            device_name = torch.cuda.get_device_name(0) if device_count > 0 else None

            checks["gpu"] = HealthCheck(
                name="gpu",
                status="healthy",
                message=f"{device_name} available",
                details={"device_count": device_count},
            )
        else:
            checks["gpu"] = HealthCheck(
                name="gpu",
                status="degraded",
                message="GPU not available, using CPU",
            )
            if overall_status == "healthy":
                overall_status = "degraded"

    except ImportError:
        checks["gpu"] = HealthCheck(
            name="gpu",
            status="degraded",
            message="PyTorch not installed",
        )
        if overall_status == "healthy":
            overall_status = "degraded"

    # Check queue status
    try:
        if service:
            stats = service.queue_manager.get_stats()
            queue_length = stats.pending_jobs

            checks["queue"] = HealthCheck(
                name="queue",
                status="healthy",
                message=f"{queue_length} pending jobs",
                details={
                    "pending": stats.pending_jobs,
                    "processing": stats.processing_jobs,
                    "completed": stats.completed_jobs,
                    "failed": stats.failed_jobs,
                },
            )
        else:
            checks["queue"] = HealthCheck(
                name="queue",
                status="unhealthy",
                message="Service not initialized",
            )
            overall_status = "unhealthy"

    except Exception as e:
        checks["queue"] = HealthCheck(
            name="queue",
            status="unhealthy",
            message=f"Failed: {str(e)}",
        )
        overall_status = "unhealthy"

    # Check disk space
    try:
        output_dir = settings.output_dir
        if output_dir.exists():
            stats = shutil.disk_usage(output_dir)
            free_gb = stats.free / (1024 ** 3)
            total_gb = stats.total / (1024 ** 3)
            percent_free = (stats.free / stats.total) * 100

            if percent_free < 10:
                status = "unhealthy"
                message = f"Low disk space: {free_gb:.1f}GB free ({percent_free:.1f}%)"
                overall_status = "unhealthy"
            elif percent_free < 20:
                status = "degraded"
                message = f"Disk space getting low: {free_gb:.1f}GB free ({percent_free:.1f}%)"
                if overall_status == "healthy":
                    overall_status = "degraded"
            else:
                status = "healthy"
                message = f"{free_gb:.1f}GB free ({percent_free:.1f}%)"

            checks["disk_space"] = HealthCheck(
                name="disk_space",
                status=status,
                message=message,
                details={
                    "free_gb": round(free_gb, 2),
                    "total_gb": round(total_gb, 2),
                    "percent_free": round(percent_free, 2),
                },
            )
        else:
            checks["disk_space"] = HealthCheck(
                name="disk_space",
                status="degraded",
                message="Output directory does not exist",
            )
            if overall_status == "healthy":
                overall_status = "degraded"

    except Exception as e:
        checks["disk_space"] = HealthCheck(
            name="disk_space",
            status="unhealthy",
            message=f"Failed: {str(e)}",
        )
        overall_status = "unhealthy"

    # Calculate uptime
    uptime = time.time() - startup_time

    return HealthStatus(
        status=overall_status,
        checks=checks,
        version=settings.version,
        uptime_seconds=uptime,
        timestamp=datetime.utcnow(),
    )


@app.get(
    "/api/v1/health/ready",
    summary="Readiness check",
    description="Simple readiness check for Kubernetes/Docker health probes.",
    tags=["Health"],
)
async def readiness_check():
    """
    Simple readiness check.

    Returns:
        Ready status
    """
    if not service:
        raise HTTPException(status_code=503, detail="Service not ready")

    return {"status": "ready"}


@app.get(
    "/api/v1/health/live",
    summary="Liveness check",
    description="Simple liveness check for Kubernetes/Docker health probes.",
    tags=["Health"],
)
async def liveness_check():
    """
    Simple liveness check.

    Returns:
        Alive status
    """
    return {"status": "alive"}


# ============================================================================
# Root Endpoint
# ============================================================================


@app.get(
    "/",
    summary="API information",
    description="Get basic API information and links to documentation.",
    tags=["Info"],
)
async def root():
    """
    Root endpoint with API information.

    Returns:
        API information
    """
    return {
        "name": "DGX Music Generation API",
        "version": settings.version,
        "docs": "/api/v1/docs",
        "health": "/api/v1/health",
        "endpoints": {
            "generate": "POST /api/v1/generate",
            "batch_generate": "POST /api/v1/generate/batch",
            "job_status": "GET /api/v1/jobs/{job_id}",
            "cancel_job": "DELETE /api/v1/jobs/{job_id}",
            "download_file": "GET /api/v1/files/{filename}",
            "queue_stats": "GET /api/v1/queue/stats",
        },
    }


# ============================================================================
# Error Handlers
# ============================================================================


@app.exception_handler(404)
async def not_found_handler(request: Request, exc: HTTPException):
    """Custom 404 handler."""
    return JSONResponse(
        status_code=404,
        content={
            "error": "Not Found",
            "message": "The requested resource was not found",
            "path": str(request.url.path),
        },
    )


@app.exception_handler(500)
async def internal_error_handler(request: Request, exc: Exception):
    """Custom 500 handler."""
    logger.error(f"Internal server error: {exc}")
    return JSONResponse(
        status_code=500,
        content={
            "error": "Internal Server Error",
            "message": "An internal server error occurred",
        },
    )


# ============================================================================
# Rate Limit Configuration
# ============================================================================


# Whitelist localhost for development
@app.middleware("http")
async def whitelist_localhost(request: Request, call_next):
    """
    Whitelist localhost from rate limiting for development.

    Args:
        request: HTTP request
        call_next: Next middleware

    Returns:
        Response
    """
    # Get client IP
    client_ip = get_remote_address(request)

    # Bypass rate limiting for localhost
    if client_ip in ["127.0.0.1", "localhost", "::1"]:
        # Temporarily disable rate limiting for this request
        request.state.view_rate_limit = None

    response = await call_next(request)
    return response
