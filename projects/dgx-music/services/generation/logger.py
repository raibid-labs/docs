"""
Logging Infrastructure for DGX Music
====================================

Centralized logging configuration with file and console output.
Supports structured logging for better observability.
"""

import logging
import sys
from pathlib import Path
from typing import Optional

from .config import settings


class ColoredFormatter(logging.Formatter):
    """
    Custom formatter with color support for console output.
    """

    # ANSI color codes
    COLORS = {
        'DEBUG': '\033[36m',      # Cyan
        'INFO': '\033[32m',       # Green
        'WARNING': '\033[33m',    # Yellow
        'ERROR': '\033[31m',      # Red
        'CRITICAL': '\033[35m',   # Magenta
        'RESET': '\033[0m'        # Reset
    }

    def format(self, record):
        # Add color to levelname
        levelname = record.levelname
        if levelname in self.COLORS:
            record.levelname = f"{self.COLORS[levelname]}{levelname}{self.COLORS['RESET']}"

        return super().format(record)


def setup_logging(
    name: str = "dgx-music",
    level: Optional[str] = None,
    log_to_file: Optional[bool] = None,
    log_to_console: Optional[bool] = None,
) -> logging.Logger:
    """
    Setup logging for the application.

    Args:
        name: Logger name
        level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
        log_to_file: Enable file logging
        log_to_console: Enable console logging

    Returns:
        Configured logger instance
    """
    # Use settings defaults if not specified
    level = level or settings.log_level
    log_to_file = log_to_file if log_to_file is not None else settings.log_to_file
    log_to_console = log_to_console if log_to_console is not None else settings.log_to_console

    # Create logger
    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, level.upper()))

    # Clear existing handlers
    logger.handlers = []

    # File handler
    if log_to_file:
        log_file = settings.log_dir / f"{name}.log"
        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(getattr(logging, level.upper()))
        file_formatter = logging.Formatter(settings.log_format)
        file_handler.setFormatter(file_formatter)
        logger.addHandler(file_handler)

    # Console handler
    if log_to_console:
        console_handler = logging.StreamHandler(sys.stdout)
        console_handler.setLevel(getattr(logging, level.upper()))
        console_formatter = ColoredFormatter(settings.log_format)
        console_handler.setFormatter(console_formatter)
        logger.addHandler(console_handler)

    # Prevent propagation to root logger
    logger.propagate = False

    return logger


# Global logger instance
logger = setup_logging()


def get_logger(name: str = "dgx-music") -> logging.Logger:
    """
    Get a logger instance.

    Args:
        name: Logger name (creates child logger if different from default)

    Returns:
        Logger instance
    """
    if name == "dgx-music":
        return logger
    return logger.getChild(name)


class LogContext:
    """
    Context manager for structured logging with additional context.

    Usage:
        with LogContext(job_id="gen_123", operation="generate"):
            logger.info("Starting generation")
            # ... do work ...
            logger.info("Completed generation")
    """

    def __init__(self, **context):
        self.context = context
        self.logger = logger

    def __enter__(self):
        # Store original log record factory
        self.old_factory = logging.getLogRecordFactory()

        # Create new factory that adds context
        def record_factory(*args, **kwargs):
            record = self.old_factory(*args, **kwargs)
            for key, value in self.context.items():
                setattr(record, key, value)
            return record

        logging.setLogRecordFactory(record_factory)
        return self.logger

    def __exit__(self, exc_type, exc_val, exc_tb):
        # Restore original factory
        logging.setLogRecordFactory(self.old_factory)


def log_performance(
    operation: str,
    duration: float,
    success: bool = True,
    **metadata
):
    """
    Log performance metrics.

    Args:
        operation: Operation name
        duration: Duration in seconds
        success: Whether operation succeeded
        **metadata: Additional metadata to log
    """
    status = "SUCCESS" if success else "FAILED"
    metadata_str = " ".join(f"{k}={v}" for k, v in metadata.items())

    logger.info(
        f"PERFORMANCE: {operation} {status} duration={duration:.2f}s {metadata_str}"
    )


def log_memory_usage():
    """
    Log current memory usage (GPU and system).
    """
    try:
        import torch
        import psutil

        # System memory
        process = psutil.Process()
        system_memory_mb = process.memory_info().rss / 1024 / 1024

        logger.debug(f"System Memory: {system_memory_mb:.2f} MB")

        # GPU memory
        if torch.cuda.is_available():
            allocated_mb = torch.cuda.memory_allocated() / 1024 / 1024
            reserved_mb = torch.cuda.memory_reserved() / 1024 / 1024
            logger.debug(
                f"GPU Memory: allocated={allocated_mb:.2f}MB reserved={reserved_mb:.2f}MB"
            )

    except ImportError:
        logger.debug("Could not import psutil or torch for memory logging")


if __name__ == "__main__":
    # Test logging
    test_logger = setup_logging("test", level="DEBUG")

    test_logger.debug("This is a debug message")
    test_logger.info("This is an info message")
    test_logger.warning("This is a warning message")
    test_logger.error("This is an error message")
    test_logger.critical("This is a critical message")

    # Test context
    with LogContext(job_id="test_123", user="test_user"):
        test_logger.info("Message with context")

    # Test performance logging
    log_performance("test_operation", 1.5, success=True, items=10)
