"""
Pytest Configuration and Shared Fixtures
=========================================

Shared fixtures for all tests in the DGX Music project.
"""

import os
import tempfile
from pathlib import Path
from typing import Generator
import pytest
import torch

from services.storage.database import init_db, get_session, reset_database
from services.generation.engine import MusicGenerationEngine
from tests.utils.mock_helpers import MockMusicGenerationEngine


# ========== Pytest Configuration ==========


def pytest_configure(config):
    """Configure pytest with custom markers."""
    config.addinivalue_line(
        "markers", "integration: mark test as integration test (may be slow)"
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow (>30 seconds)"
    )
    config.addinivalue_line(
        "markers", "gpu: mark test as requiring GPU/CUDA"
    )
    config.addinivalue_line(
        "markers", "e2e: mark test as end-to-end test"
    )


# ========== Directory Fixtures ==========


@pytest.fixture
def temp_dir(tmp_path) -> Path:
    """Create a temporary directory for test artifacts."""
    test_dir = tmp_path / "dgx_music_test"
    test_dir.mkdir(exist_ok=True)
    return test_dir


@pytest.fixture
def output_dir(temp_dir) -> Path:
    """Create a temporary output directory for generated audio."""
    outputs = temp_dir / "outputs"
    outputs.mkdir(exist_ok=True)
    return outputs


@pytest.fixture
def data_dir(temp_dir) -> Path:
    """Create a temporary data directory for databases."""
    data = temp_dir / "data"
    data.mkdir(exist_ok=True)
    return data


# ========== Database Fixtures ==========


@pytest.fixture
def test_db_url(data_dir) -> str:
    """Create a test database URL."""
    db_path = data_dir / "test.db"
    return f"sqlite:///{db_path}"


@pytest.fixture
def db_session(test_db_url):
    """
    Create a database session for testing.

    This fixture initializes a test database and provides a session.
    The database is cleaned up after the test.
    """
    # Initialize database
    init_db(test_db_url)

    # Provide session
    with get_session() as session:
        yield session

    # Cleanup is handled by get_session context manager


@pytest.fixture
def clean_db_session(test_db_url):
    """
    Create a clean database session (resets database before test).

    Use this when you need a completely fresh database state.
    """
    # Initialize and reset database
    init_db(test_db_url)
    reset_database()

    # Provide session
    with get_session() as session:
        yield session


# ========== Generation Engine Fixtures ==========


@pytest.fixture
def mock_engine() -> MockMusicGenerationEngine:
    """
    Create a mock generation engine for fast testing.

    This engine doesn't require GPU and generates simple test audio.
    Use this for most integration tests that don't need real generation.
    """
    return MockMusicGenerationEngine(
        model_name="small",
        use_gpu=False,
        enable_caching=False,
        generation_delay=0.1,  # Fast for testing
    )


@pytest.fixture(scope="module")
def real_engine():
    """
    Create a real generation engine (requires GPU).

    This fixture is scoped to module to avoid repeated model loading.
    Tests using this fixture will be skipped if CUDA is not available.
    """
    if not torch.cuda.is_available():
        pytest.skip("CUDA not available - skipping GPU tests")

    engine = MusicGenerationEngine(
        model_name="small",
        use_gpu=True,
        enable_caching=True,
    )

    yield engine

    # Cleanup
    engine.unload_model()


@pytest.fixture
def mock_settings(output_dir, monkeypatch):
    """
    Mock settings for generation engine.

    This fixture patches the settings to use test directories.
    """
    class MockSettings:
        normalize_audio = True
        model_name = "musicgen-small"
        use_gpu = False
        enable_model_caching = False

        def get_output_path(self, job_id: str) -> Path:
            return output_dir / f"{job_id}.wav"

    # Patch settings
    from services.generation import engine as engine_module
    monkeypatch.setattr(engine_module, "settings", MockSettings())

    return MockSettings()


# ========== Audio Testing Fixtures ==========


@pytest.fixture
def test_audio_file(output_dir) -> Path:
    """
    Create a test audio file.

    Returns path to a valid WAV file for testing.
    """
    from tests.utils.audio_helpers import generate_test_wav

    file_path = output_dir / "test_audio.wav"
    generate_test_wav(file_path, duration=2.0, sample_rate=32000, channels=2)

    return file_path


@pytest.fixture
def test_audio_tensor() -> torch.Tensor:
    """
    Create a test audio tensor.

    Returns a stereo audio tensor for testing.
    """
    from tests.utils.audio_helpers import create_test_audio_tensor

    return create_test_audio_tensor(
        duration=2.0,
        sample_rate=32000,
        channels=2,
        frequency=440.0,
    )


# ========== Seeded Data Fixtures ==========


@pytest.fixture
def seeded_db_session(clean_db_session):
    """
    Create a database session with seeded test data.

    Includes:
    - 10 test generations (various statuses)
    - 5 test prompts (with usage tracking)
    """
    from tests.utils.db_helpers import seed_test_generations, seed_test_prompts

    session = clean_db_session

    # Seed data
    generations = seed_test_generations(session, count=10)
    prompts = seed_test_prompts(session, count=5)

    yield session

    # Cleanup files
    from tests.utils.db_helpers import cleanup_test_files
    cleanup_test_files(generations)


# ========== Performance Measurement Fixtures ==========


@pytest.fixture
def performance_tracker():
    """
    Track performance metrics during tests.

    Usage:
        def test_something(performance_tracker):
            with performance_tracker.measure("operation"):
                # ... do something

            assert performance_tracker.get_duration("operation") < 1.0
    """
    import time

    class PerformanceTracker:
        def __init__(self):
            self.measurements = {}
            self.current_operation = None
            self.start_time = None

        def measure(self, operation: str):
            """Context manager for measuring operation time."""
            class MeasureContext:
                def __init__(ctx_self):
                    ctx_self.operation = operation

                def __enter__(ctx_self):
                    self.current_operation = operation
                    self.start_time = time.time()
                    return ctx_self

                def __exit__(ctx_self, *args):
                    duration = time.time() - self.start_time
                    self.measurements[operation] = duration
                    self.current_operation = None
                    self.start_time = None

            return MeasureContext()

        def get_duration(self, operation: str) -> float:
            """Get duration of an operation."""
            return self.measurements.get(operation, 0.0)

        def get_all_measurements(self) -> dict:
            """Get all measurements."""
            return self.measurements.copy()

    return PerformanceTracker()


# ========== Environment Fixtures ==========


@pytest.fixture
def mock_cuda_available(monkeypatch):
    """
    Mock CUDA availability for testing without GPU.

    Usage:
        def test_something(mock_cuda_available):
            mock_cuda_available(True)  # Pretend CUDA is available
    """
    def _mock(available: bool):
        monkeypatch.setattr("torch.cuda.is_available", lambda: available)
        monkeypatch.setattr("torch.cuda.get_device_name", lambda x: "Mock GPU")

    return _mock


@pytest.fixture
def mock_no_pyloudnorm(monkeypatch):
    """Mock missing pyloudnorm dependency."""
    monkeypatch.setattr("services.audio.export.PYLOUDNORM_AVAILABLE", False)


@pytest.fixture
def mock_no_librosa(monkeypatch):
    """Mock missing librosa dependency."""
    monkeypatch.setattr("services.audio.metadata.LIBROSA_AVAILABLE", False)


# ========== Cleanup Fixtures ==========


@pytest.fixture(autouse=True)
def cleanup_outputs(output_dir):
    """
    Automatically clean up output directory after each test.

    This fixture runs after every test to ensure a clean state.
    """
    yield

    # Cleanup after test
    if output_dir.exists():
        for file in output_dir.glob("*.wav"):
            try:
                file.unlink()
            except Exception:
                pass  # Best effort cleanup


# ========== Integration Test Helpers ==========


@pytest.fixture
def integration_setup(clean_db_session, output_dir, mock_settings):
    """
    Complete setup for integration tests.

    Provides:
    - Clean database session
    - Output directory
    - Mocked settings
    - Helper functions

    Returns a namespace object with all components.
    """
    class IntegrationSetup:
        def __init__(self):
            self.db_session = clean_db_session
            self.output_dir = output_dir
            self.settings = mock_settings

        def create_test_generation(self, **kwargs):
            """Create a test generation with default values."""
            from tests.utils.db_helpers import create_test_generation
            return create_test_generation(self.db_session, **kwargs)

        def create_generation_with_file(self, prompt: str, file_path: Path):
            """Create a generation with its corresponding file."""
            from tests.utils.db_helpers import create_generation_with_file
            return create_generation_with_file(
                self.db_session, prompt, file_path
            )

        def verify_consistency(self):
            """Verify database consistency."""
            from tests.utils.db_helpers import verify_database_consistency
            return verify_database_consistency(self.db_session)

    return IntegrationSetup()


# ========== Benchmark Fixtures ==========


@pytest.fixture
def benchmark_results(temp_dir):
    """
    Collect benchmark results across tests.

    Results are written to a JSON file in the temp directory.
    """
    results = {
        "tests": [],
        "timestamp": None,
    }

    def add_result(test_name: str, metrics: dict):
        """Add a benchmark result."""
        results["tests"].append({
            "name": test_name,
            "metrics": metrics,
        })

    yield add_result

    # Write results to file
    import json
    import datetime

    results["timestamp"] = datetime.datetime.now().isoformat()

    results_file = temp_dir / "benchmark_results.json"
    with open(results_file, "w") as f:
        json.dump(results, f, indent=2)


# ========== Pytest Hooks ==========


def pytest_runtest_setup(item):
    """Hook that runs before each test."""
    # Skip GPU tests if CUDA not available
    if item.get_closest_marker("gpu"):
        if not torch.cuda.is_available():
            pytest.skip("CUDA not available - skipping GPU test")


def pytest_runtest_teardown(item):
    """Hook that runs after each test."""
    # Clear GPU cache if test used GPU
    if item.get_closest_marker("gpu"):
        if torch.cuda.is_available():
            torch.cuda.empty_cache()
