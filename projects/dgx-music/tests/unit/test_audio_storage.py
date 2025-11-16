"""
Unit tests for AudioFileManager

Tests cover:
- Initialization
- Path generation with date-based organization
- File operations (move, copy, delete)
- Cleanup utilities
- Storage statistics
- File listing
- Error handling
"""

import pytest
import shutil
import time
from pathlib import Path
from datetime import datetime, timedelta
import tempfile

from services.audio.storage import AudioFileManager


@pytest.fixture
def temp_dir():
    """Create a temporary base directory for tests."""
    temp_path = tempfile.mkdtemp()
    yield Path(temp_path)
    shutil.rmtree(temp_path, ignore_errors=True)


@pytest.fixture
def manager(temp_dir):
    """Create an AudioFileManager instance with temp directory."""
    return AudioFileManager(base_dir=str(temp_dir))


@pytest.fixture
def sample_files(manager):
    """Create sample test files."""
    files = []

    # Create files for today
    for i in range(3):
        path = manager.get_output_path(f"gen_today_{i}", create_dirs=True)
        path.write_text(f"test content {i}")
        files.append(path)

    # Create files for yesterday
    yesterday = datetime.now() - timedelta(days=1)
    for i in range(2):
        path = manager.get_output_path(f"gen_yesterday_{i}", date=yesterday, create_dirs=True)
        path.write_text(f"old content {i}")
        files.append(path)

    return files


# ========== Initialization Tests ==========


def test_manager_initialization(temp_dir):
    """Test AudioFileManager initialization."""
    manager = AudioFileManager(base_dir=str(temp_dir))
    assert manager.base_dir == temp_dir
    assert temp_dir.exists()


def test_manager_creates_base_dir():
    """Test that base directory is created if it doesn't exist."""
    temp_path = Path(tempfile.mkdtemp())
    base_dir = temp_path / "new_base_dir"

    # Ensure it doesn't exist
    if base_dir.exists():
        shutil.rmtree(base_dir)

    manager = AudioFileManager(base_dir=str(base_dir))
    assert base_dir.exists()

    # Cleanup
    shutil.rmtree(temp_path)


# ========== Path Generation Tests ==========


def test_get_output_path_default(manager):
    """Test output path generation with defaults."""
    path = manager.get_output_path("gen_test123")

    # Should create date-based path
    today = datetime.now()
    expected_pattern = f"{today.year:04d}/{today.month:02d}/{today.day:02d}"
    assert expected_pattern in str(path)
    assert path.name == "gen_test123.wav"


def test_get_output_path_custom_extension(manager):
    """Test output path with custom extension."""
    path = manager.get_output_path("gen_test", extension=".flac")
    assert path.suffix == ".flac"


def test_get_output_path_custom_date(manager):
    """Test output path with custom date."""
    custom_date = datetime(2025, 1, 15)
    path = manager.get_output_path("gen_custom", date=custom_date)

    assert "2025/01/15" in str(path)
    assert path.name == "gen_custom.wav"


def test_get_output_path_creates_directories(manager):
    """Test that get_output_path creates directories."""
    path = manager.get_output_path("gen_test", create_dirs=True)
    assert path.parent.exists()


def test_get_output_path_no_create_dirs(manager):
    """Test that get_output_path can skip directory creation."""
    path = manager.get_output_path("gen_test", create_dirs=False)
    # Parent might not exist
    assert isinstance(path, Path)


def test_output_path_structure(manager):
    """Test that output path follows YYYY/MM/DD structure."""
    today = datetime.now()
    path = manager.get_output_path("test_job")

    parts = path.relative_to(manager.base_dir).parts
    assert len(parts) == 4  # YYYY, MM, DD, filename
    assert parts[0] == f"{today.year:04d}"
    assert parts[1] == f"{today.month:02d}"
    assert parts[2] == f"{today.day:02d}"
    assert parts[3] == "test_job.wav"


# ========== File Operations Tests ==========


def test_get_file_size(manager, sample_files):
    """Test getting file size in bytes."""
    test_file = sample_files[0]
    size = manager.get_file_size(test_file)

    assert size > 0
    assert size == test_file.stat().st_size


def test_get_file_size_mb(manager, sample_files):
    """Test getting file size in MB."""
    test_file = sample_files[0]
    size_bytes = test_file.stat().st_size
    size_mb = manager.get_file_size_mb(test_file)

    expected_mb = size_bytes / (1024 * 1024)
    assert size_mb == pytest.approx(expected_mb)


def test_get_file_size_nonexistent(manager):
    """Test that getting size of nonexistent file raises error."""
    with pytest.raises(FileNotFoundError):
        manager.get_file_size("nonexistent.wav")


def test_file_exists(manager, sample_files):
    """Test checking if file exists."""
    # File that exists
    path = manager.get_output_path("gen_today_0", create_dirs=False)
    assert manager.file_exists("gen_today_0")

    # File that doesn't exist
    assert not manager.file_exists("gen_nonexistent")


def test_file_exists_with_custom_date(manager, sample_files):
    """Test file exists check with custom date."""
    yesterday = datetime.now() - timedelta(days=1)
    assert manager.file_exists("gen_yesterday_0", date=yesterday)


def test_delete_file(manager, sample_files):
    """Test deleting a file."""
    test_file = sample_files[0]
    assert test_file.exists()

    result = manager.delete_file(test_file)
    assert result is True
    assert not test_file.exists()


def test_delete_nonexistent_file(manager):
    """Test deleting nonexistent file returns False."""
    result = manager.delete_file("nonexistent.wav")
    assert result is False


def test_move_file(manager, sample_files):
    """Test moving a file."""
    source = sample_files[0]
    destination = manager.base_dir / "moved" / "test.wav"

    moved_path = manager.move_file(source, destination)

    assert not source.exists()
    assert moved_path.exists()
    assert moved_path == destination


def test_move_nonexistent_file(manager):
    """Test moving nonexistent file raises error."""
    with pytest.raises(FileNotFoundError):
        manager.move_file("nonexistent.wav", "destination.wav")


def test_copy_file(manager, sample_files):
    """Test copying a file."""
    source = sample_files[0]
    destination = manager.base_dir / "copied" / "test.wav"

    copied_path = manager.copy_file(source, destination)

    assert source.exists()  # Original still exists
    assert copied_path.exists()  # Copy exists
    assert copied_path == destination


def test_copy_nonexistent_file(manager):
    """Test copying nonexistent file raises error."""
    with pytest.raises(FileNotFoundError):
        manager.copy_file("nonexistent.wav", "destination.wav")


# ========== Cleanup Tests ==========


def test_cleanup_old_files_dry_run(manager, sample_files):
    """Test cleanup with dry run (no actual deletion)."""
    # All files are recent, so nothing should be deleted
    count = manager.cleanup_old_files(days_old=30, dry_run=True)
    assert count == 0

    # All files should still exist
    for f in sample_files:
        assert f.exists()


def test_cleanup_old_files_actual_delete(manager, temp_dir):
    """Test actual deletion of old files."""
    # Create an old file by modifying its timestamp
    old_file = manager.get_output_path("gen_old", create_dirs=True)
    old_file.write_text("old content")

    # Set modification time to 60 days ago
    old_time = time.time() - (60 * 24 * 60 * 60)
    import os
    os.utime(old_file, (old_time, old_time))

    # Cleanup files older than 30 days
    count = manager.cleanup_old_files(days_old=30, dry_run=False)
    assert count == 1
    assert not old_file.exists()


def test_cleanup_preserves_recent_files(manager, sample_files):
    """Test that cleanup preserves recent files."""
    initial_count = len(list(manager.base_dir.rglob("*")))

    # Cleanup files older than 1 day (should preserve today's files)
    manager.cleanup_old_files(days_old=1, dry_run=False)

    # Today's files should still exist
    today_files = [f for f in sample_files if "today" in f.name]
    for f in today_files:
        assert f.exists()


def test_cleanup_empty_directories(manager, temp_dir):
    """Test removing empty directories."""
    # Create nested empty directories
    empty_dir = manager.base_dir / "2025" / "01" / "01"
    empty_dir.mkdir(parents=True, exist_ok=True)

    # Another empty directory
    empty_dir2 = manager.base_dir / "2025" / "02" / "02"
    empty_dir2.mkdir(parents=True, exist_ok=True)

    # Cleanup
    count = manager.cleanup_empty_directories()
    assert count >= 2  # At least the two empty leaf directories


def test_cleanup_preserves_non_empty_directories(manager, sample_files):
    """Test that cleanup preserves directories with files."""
    # Get directory containing files
    file_dir = sample_files[0].parent

    # Cleanup empty directories
    manager.cleanup_empty_directories()

    # Directory with files should still exist
    assert file_dir.exists()


# ========== Storage Statistics Tests ==========


def test_get_storage_stats_empty(manager):
    """Test storage stats for empty storage."""
    stats = manager.get_storage_stats()

    assert stats['total_files'] == 0
    assert stats['total_size_bytes'] == 0
    assert stats['total_size_mb'] == 0.0
    assert stats['total_size_gb'] == 0.0
    assert stats['oldest_file'] is None
    assert stats['newest_file'] is None
    assert stats['file_types'] == {}


def test_get_storage_stats_with_files(manager, sample_files):
    """Test storage stats with files."""
    stats = manager.get_storage_stats()

    assert stats['total_files'] == len(sample_files)
    assert stats['total_size_bytes'] > 0
    assert stats['total_size_mb'] > 0
    assert stats['oldest_file'] is not None
    assert stats['newest_file'] is not None
    assert '.wav' in stats['file_types']


def test_storage_stats_file_types(manager, temp_dir):
    """Test that file types are correctly counted."""
    # Create files with different extensions
    wav_path = manager.get_output_path("test", extension=".wav", create_dirs=True)
    wav_path.write_text("wav content")

    flac_path = manager.get_output_path("test", extension=".flac", create_dirs=True)
    flac_path.write_text("flac content")

    stats = manager.get_storage_stats()

    assert stats['file_types']['.wav'] >= 1
    assert stats['file_types']['.flac'] >= 1


# ========== File Listing Tests ==========


def test_list_files_all(manager, sample_files):
    """Test listing all files."""
    files = manager.list_files()
    assert len(files) == len(sample_files)


def test_list_files_with_limit(manager, sample_files):
    """Test listing files with limit."""
    files = manager.list_files(limit=2)
    assert len(files) == 2


def test_list_files_sorted_by_time(manager, sample_files):
    """Test that files are sorted by modification time (newest first)."""
    files = manager.list_files()

    # Get modification times
    mtimes = [f.stat().st_mtime for f in files]

    # Should be in descending order (newest first)
    assert mtimes == sorted(mtimes, reverse=True)


def test_list_files_by_extension(manager, temp_dir):
    """Test listing files filtered by extension."""
    # Create files with different extensions
    wav_path = manager.get_output_path("test1", extension=".wav", create_dirs=True)
    wav_path.write_text("wav")

    flac_path = manager.get_output_path("test2", extension=".flac", create_dirs=True)
    flac_path.write_text("flac")

    # List only WAV files
    wav_files = manager.list_files(extension=".wav")
    assert all(f.suffix == ".wav" for f in wav_files)


def test_list_files_by_date_range(manager, sample_files):
    """Test listing files by date range."""
    today = datetime.now()
    yesterday = today - timedelta(days=1)
    tomorrow = today + timedelta(days=1)

    # List files from yesterday to tomorrow (should include all)
    files = manager.list_files(start_date=yesterday, end_date=tomorrow)
    assert len(files) > 0


def test_list_files_empty_result(manager):
    """Test listing files when none match criteria."""
    # List files from far future
    future = datetime.now() + timedelta(days=365)
    files = manager.list_files(start_date=future)
    assert len(files) == 0


# ========== Configuration Tests ==========


def test_get_info(manager):
    """Test getting manager configuration info."""
    info = manager.get_info()

    assert 'base_dir' in info
    assert 'base_dir_exists' in info
    assert info['base_dir_exists'] is True


# ========== Edge Cases Tests ==========


def test_path_with_extension_already_present(manager):
    """Test path generation when job_id already has extension."""
    path = manager.get_output_path("gen_test.wav", extension=".wav")
    # Should not duplicate extension
    assert path.name == "gen_test.wav"


def test_unicode_in_job_id(manager):
    """Test handling of unicode characters in job_id."""
    # This should work or raise a clear error
    path = manager.get_output_path("gen_test_音楽")
    assert isinstance(path, Path)


def test_very_long_job_id(manager):
    """Test handling of very long job_id."""
    long_id = "gen_" + "x" * 200
    path = manager.get_output_path(long_id)
    assert isinstance(path, Path)


# ========== Integration Tests ==========


def test_complete_file_lifecycle(manager):
    """Test complete file lifecycle: create, move, copy, delete."""
    # Create file
    original_path = manager.get_output_path("gen_lifecycle", create_dirs=True)
    original_path.write_text("test content")
    assert original_path.exists()

    # Copy file
    copy_path = manager.base_dir / "backup" / "copy.wav"
    copied = manager.copy_file(original_path, copy_path)
    assert original_path.exists()
    assert copied.exists()

    # Move original
    move_path = manager.base_dir / "archive" / "moved.wav"
    moved = manager.move_file(original_path, move_path)
    assert not original_path.exists()
    assert moved.exists()

    # Delete both
    assert manager.delete_file(copied)
    assert manager.delete_file(moved)
    assert not copied.exists()
    assert not moved.exists()


def test_bulk_operations(manager):
    """Test bulk file operations."""
    # Create multiple files
    paths = []
    for i in range(10):
        path = manager.get_output_path(f"gen_bulk_{i}", create_dirs=True)
        path.write_text(f"content {i}")
        paths.append(path)

    # Verify all created
    assert len(manager.list_files()) == 10

    # Delete half
    for path in paths[:5]:
        manager.delete_file(path)

    # Verify remaining
    assert len(manager.list_files()) == 5

    # Cleanup rest
    for path in paths[5:]:
        manager.delete_file(path)

    assert len(manager.list_files()) == 0
