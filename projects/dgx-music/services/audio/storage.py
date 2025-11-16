"""
Audio File Storage Management Module
======================================

Manage audio file storage with organized directory structure.

This module provides the AudioFileManager class which handles:
- Date-based organization: data/outputs/YYYY/MM/DD/job_id.wav
- Automatic directory creation
- File operations: move, copy, delete
- Cleanup utilities: delete old files, remove empty directories
- Storage statistics and file listing

Usage:
    from services.audio import AudioFileManager

    manager = AudioFileManager()
    output_path = manager.get_output_path("gen_abc123")
    # Returns: data/outputs/2025/11/07/gen_abc123.wav
"""

import logging
import shutil
import time
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Dict, Any, Optional, Union

logger = logging.getLogger(__name__)


class AudioFileManager:
    """
    Manage audio file storage with date-based organization.

    This class handles all file management operations:
    - Automatic directory creation in YYYY/MM/DD structure
    - Path generation for new files
    - File operations (move, copy, delete)
    - Cleanup utilities (old files, empty directories)
    - Storage statistics and listing

    Attributes:
        base_dir (Path): Base directory for all audio files
    """

    def __init__(self, base_dir: str = "data/outputs"):
        """
        Initialize the AudioFileManager.

        Args:
            base_dir: Base directory for audio file storage.
                     Files will be organized as base_dir/YYYY/MM/DD/
        """
        self.base_dir = Path(base_dir)
        logger.info(f"AudioFileManager initialized: base_dir={self.base_dir}")

        # Create base directory if it doesn't exist
        self.base_dir.mkdir(parents=True, exist_ok=True)

    def get_output_path(
        self,
        job_id: str,
        extension: str = ".wav",
        create_dirs: bool = True,
        date: Optional[datetime] = None,
    ) -> Path:
        """
        Get the output path for a job ID with date-based organization.

        Creates directory structure: base_dir/YYYY/MM/DD/job_id.ext

        Args:
            job_id: Unique job identifier (will be used as filename)
            extension: File extension (default: .wav)
            create_dirs: Create parent directories if they don't exist
            date: Date to use for path (default: today)

        Returns:
            Path object for the output file
        """
        # Use provided date or today
        if date is None:
            date = datetime.now()

        # Build date-based path
        year = f"{date.year:04d}"
        month = f"{date.month:02d}"
        day = f"{date.day:02d}"

        # Create directory path
        dir_path = self.base_dir / year / month / day

        # Create directories if requested
        if create_dirs:
            dir_path.mkdir(parents=True, exist_ok=True)
            logger.debug(f"Created directory: {dir_path}")

        # Add extension if not present
        if not job_id.endswith(extension):
            filename = job_id + extension
        else:
            filename = job_id

        output_path = dir_path / filename
        logger.debug(f"Generated output path: {output_path}")

        return output_path

    def get_file_size(self, path: Union[str, Path]) -> int:
        """
        Get file size in bytes.

        Args:
            path: Path to file

        Returns:
            File size in bytes

        Raises:
            FileNotFoundError: If file doesn't exist
        """
        path = Path(path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {path}")

        return path.stat().st_size

    def get_file_size_mb(self, path: Union[str, Path]) -> float:
        """
        Get file size in megabytes.

        Args:
            path: Path to file

        Returns:
            File size in MB (float)
        """
        size_bytes = self.get_file_size(path)
        return size_bytes / (1024 * 1024)

    def file_exists(
        self,
        job_id: str,
        extension: str = ".wav",
        date: Optional[datetime] = None,
    ) -> bool:
        """
        Check if a file exists for a given job ID.

        Args:
            job_id: Job identifier
            extension: File extension
            date: Date to check (default: today)

        Returns:
            True if file exists, False otherwise
        """
        path = self.get_output_path(job_id, extension, create_dirs=False, date=date)
        return path.exists()

    def delete_file(self, path: Union[str, Path]) -> bool:
        """
        Delete a file.

        Args:
            path: Path to file

        Returns:
            True if deleted, False if file didn't exist

        Raises:
            PermissionError: If file can't be deleted
        """
        path = Path(path)

        if not path.exists():
            logger.warning(f"Cannot delete non-existent file: {path}")
            return False

        try:
            path.unlink()
            logger.info(f"Deleted file: {path}")
            return True
        except Exception as e:
            logger.error(f"Failed to delete file {path}: {e}")
            raise

    def move_file(
        self,
        source: Union[str, Path],
        destination: Union[str, Path],
    ) -> Path:
        """
        Move a file to a new location.

        Args:
            source: Source file path
            destination: Destination file path

        Returns:
            Path to moved file

        Raises:
            FileNotFoundError: If source doesn't exist
            RuntimeError: If move fails
        """
        source = Path(source)
        destination = Path(destination)

        if not source.exists():
            raise FileNotFoundError(f"Source file not found: {source}")

        # Create destination directory
        destination.parent.mkdir(parents=True, exist_ok=True)

        try:
            shutil.move(str(source), str(destination))
            logger.info(f"Moved file: {source} → {destination}")
            return destination
        except Exception as e:
            logger.error(f"Failed to move file: {e}")
            raise RuntimeError(f"Could not move file: {e}")

    def copy_file(
        self,
        source: Union[str, Path],
        destination: Union[str, Path],
    ) -> Path:
        """
        Copy a file to a new location.

        Args:
            source: Source file path
            destination: Destination file path

        Returns:
            Path to copied file

        Raises:
            FileNotFoundError: If source doesn't exist
            RuntimeError: If copy fails
        """
        source = Path(source)
        destination = Path(destination)

        if not source.exists():
            raise FileNotFoundError(f"Source file not found: {source}")

        # Create destination directory
        destination.parent.mkdir(parents=True, exist_ok=True)

        try:
            shutil.copy2(str(source), str(destination))
            logger.info(f"Copied file: {source} → {destination}")
            return destination
        except Exception as e:
            logger.error(f"Failed to copy file: {e}")
            raise RuntimeError(f"Could not copy file: {e}")

    def cleanup_old_files(
        self,
        days_old: int = 30,
        dry_run: bool = True,
    ) -> int:
        """
        Delete files older than specified number of days.

        Args:
            days_old: Delete files older than this many days
            dry_run: If True, only report what would be deleted (don't delete)

        Returns:
            Number of files deleted (or would be deleted in dry run)
        """
        logger.info(
            f"Cleaning up files older than {days_old} days "
            f"(dry_run={dry_run})"
        )

        cutoff_time = time.time() - (days_old * 24 * 60 * 60)
        deleted_count = 0

        # Find all files
        for file_path in self.base_dir.rglob("*"):
            if not file_path.is_file():
                continue

            # Check modification time
            mtime = file_path.stat().st_mtime
            if mtime < cutoff_time:
                if dry_run:
                    logger.info(f"Would delete: {file_path}")
                    deleted_count += 1
                else:
                    try:
                        file_path.unlink()
                        logger.info(f"Deleted: {file_path}")
                        deleted_count += 1
                    except Exception as e:
                        logger.error(f"Failed to delete {file_path}: {e}")

        logger.info(
            f"Cleanup complete: {deleted_count} files "
            f"{'would be' if dry_run else ''} deleted"
        )
        return deleted_count

    def cleanup_empty_directories(self) -> int:
        """
        Remove empty directories in the storage tree.

        Returns:
            Number of directories removed
        """
        logger.info("Cleaning up empty directories")
        removed_count = 0

        # Walk bottom-up to handle nested empty directories
        for dir_path in sorted(self.base_dir.rglob("*"), reverse=True):
            if not dir_path.is_dir():
                continue

            # Don't delete base directory
            if dir_path == self.base_dir:
                continue

            # Check if empty
            try:
                if not any(dir_path.iterdir()):
                    dir_path.rmdir()
                    logger.debug(f"Removed empty directory: {dir_path}")
                    removed_count += 1
            except Exception as e:
                logger.error(f"Failed to remove directory {dir_path}: {e}")

        logger.info(f"Removed {removed_count} empty directories")
        return removed_count

    def get_storage_stats(self) -> Dict[str, Any]:
        """
        Get storage statistics.

        Returns:
            Dictionary with:
                - total_files: Total number of files
                - total_size_bytes: Total size in bytes
                - total_size_mb: Total size in MB
                - total_size_gb: Total size in GB
                - oldest_file: Path to oldest file
                - newest_file: Path to newest file
                - file_types: Count of files by extension
        """
        logger.debug("Computing storage statistics")

        total_files = 0
        total_size = 0
        oldest_file = None
        oldest_time = float('inf')
        newest_file = None
        newest_time = 0
        file_types: Dict[str, int] = {}

        for file_path in self.base_dir.rglob("*"):
            if not file_path.is_file():
                continue

            total_files += 1
            total_size += file_path.stat().st_size

            # Track oldest and newest
            mtime = file_path.stat().st_mtime
            if mtime < oldest_time:
                oldest_time = mtime
                oldest_file = file_path
            if mtime > newest_time:
                newest_time = mtime
                newest_file = file_path

            # Count file types
            ext = file_path.suffix.lower()
            file_types[ext] = file_types.get(ext, 0) + 1

        return {
            "total_files": total_files,
            "total_size_bytes": total_size,
            "total_size_mb": total_size / (1024 * 1024),
            "total_size_gb": total_size / (1024 * 1024 * 1024),
            "oldest_file": str(oldest_file) if oldest_file else None,
            "newest_file": str(newest_file) if newest_file else None,
            "file_types": file_types,
        }

    def list_files(
        self,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        limit: Optional[int] = None,
        extension: Optional[str] = None,
    ) -> List[Path]:
        """
        List files with optional filtering.

        Args:
            start_date: Include files from this date onwards
            end_date: Include files up to this date
            limit: Maximum number of files to return
            extension: Filter by extension (e.g., ".wav")

        Returns:
            List of Path objects, sorted by modification time (newest first)
        """
        logger.debug(
            f"Listing files: start={start_date}, end={end_date}, "
            f"limit={limit}, ext={extension}"
        )

        files = []

        for file_path in self.base_dir.rglob("*"):
            if not file_path.is_file():
                continue

            # Filter by extension
            if extension and file_path.suffix.lower() != extension.lower():
                continue

            # Filter by date
            mtime = datetime.fromtimestamp(file_path.stat().st_mtime)

            if start_date and mtime < start_date:
                continue
            if end_date and mtime > end_date:
                continue

            files.append(file_path)

        # Sort by modification time (newest first)
        files.sort(key=lambda p: p.stat().st_mtime, reverse=True)

        # Apply limit
        if limit:
            files = files[:limit]

        logger.debug(f"Found {len(files)} files")
        return files

    def get_info(self) -> dict:
        """
        Get file manager configuration information.

        Returns:
            Dictionary with manager settings
        """
        return {
            "base_dir": str(self.base_dir),
            "base_dir_exists": self.base_dir.exists(),
        }


if __name__ == "__main__":
    # Quick test
    print("AudioFileManager module")
    print("=" * 50)

    manager = AudioFileManager()
    print(f"Configuration: {manager.get_info()}")

    # Test path generation
    print("\nGenerating output paths:")
    for job_id in ["gen_test1", "gen_test2", "gen_test3"]:
        path = manager.get_output_path(job_id, create_dirs=False)
        print(f"  {job_id}: {path}")

    # Test with custom date
    custom_date = datetime(2025, 1, 15)
    path = manager.get_output_path("gen_custom", create_dirs=False, date=custom_date)
    print(f"\nCustom date path: {path}")

    # Get storage stats
    print("\nStorage statistics:")
    stats = manager.get_storage_stats()
    for key, value in stats.items():
        if isinstance(value, float):
            print(f"  {key}: {value:.2f}")
        else:
            print(f"  {key}: {value}")
