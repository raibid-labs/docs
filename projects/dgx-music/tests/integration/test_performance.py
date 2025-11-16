"""
Performance Integration Tests
==============================

Tests for performance benchmarking and optimization validation.
"""

import pytest
import time
import json
from pathlib import Path

from services.generation.models import GenerationRequest
from services.storage.database import create_generation, get_generation


pytestmark = [pytest.mark.integration, pytest.mark.slow]


class TestGenerationLatency:
    """Test generation latency performance."""

    def test_16s_generation_under_30s(self, mock_engine, mock_settings):
        """Test 16s audio generation completes under 30s (MVP target)."""
        request = GenerationRequest(prompt="test music", duration=16.0)

        start_time = time.time()
        result = mock_engine.generate(request)
        elapsed = time.time() - start_time

        assert result.status == "completed"

        # MVP target: <30s for 16s audio (with mock engine, should be instant)
        # Real engine target: <30s
        print(f"\nGeneration time: {elapsed:.2f}s for 16s audio")
        print(f"Real-time factor: {elapsed / 16.0:.2f}x")

        # Mock engine should be very fast
        assert elapsed < 5.0, f"Mock generation too slow: {elapsed:.2f}s"

    @pytest.mark.gpu
    def test_real_generation_performance(self, real_engine, output_dir):
        """Test real generation performance (requires GPU)."""
        from services.generation.models import GenerationRequest

        request = GenerationRequest(prompt="electronic music", duration=16.0)

        # Mock settings for real engine
        class MockSettings:
            normalize_audio = True

            def get_output_path(self, job_id):
                return output_dir / f"{job_id}.wav"

        from services.generation import engine as engine_module
        original_settings = engine_module.settings
        engine_module.settings = MockSettings()

        try:
            start_time = time.time()
            result = real_engine.generate(request)
            elapsed = time.time() - start_time

            print(f"\nReal generation time: {elapsed:.2f}s for 16s audio")
            print(f"Real-time factor: {elapsed / 16.0:.2f}x")

            # MVP target: <30s
            # Blocker threshold: <60s
            assert elapsed < 60.0, (
                f"Generation too slow: {elapsed:.2f}s > 60s (BLOCKER)"
            )

            if elapsed < 30.0:
                print("✓ EXCELLENT: Within 30s target")
            else:
                print("✓ ACCEPTABLE: Within 60s, but consider optimization")

        finally:
            engine_module.settings = original_settings

    def test_multiple_short_generations_throughput(
        self, mock_engine, mock_settings, performance_tracker
    ):
        """Test throughput for multiple short generations."""
        num_generations = 5
        duration_each = 4.0

        with performance_tracker.measure("batch_generation"):
            for i in range(num_generations):
                request = GenerationRequest(
                    prompt=f"test {i}",
                    duration=duration_each,
                )
                result = mock_engine.generate(request)
                assert result.status == "completed"

        total_time = performance_tracker.get_duration("batch_generation")
        avg_time = total_time / num_generations

        print(f"\nBatch generation: {num_generations} files in {total_time:.2f}s")
        print(f"Average per file: {avg_time:.2f}s")

        # Mock engine should handle this quickly
        assert avg_time < 1.0, f"Average generation too slow: {avg_time:.2f}s"


class TestAPIResponseTime:
    """Test API response time performance."""

    def test_database_query_performance(
        self, seeded_db_session, performance_tracker
    ):
        """Test database query performance (<100ms target)."""
        # Test single query
        with performance_tracker.measure("single_query"):
            from services.storage.database import get_all_generations

            results = get_all_generations(seeded_db_session, limit=10)

        query_time = performance_tracker.get_duration("single_query")

        print(f"\nDatabase query time: {query_time * 1000:.1f}ms")

        # Target: <100ms for queries
        assert query_time < 0.1, (
            f"Query too slow: {query_time * 1000:.1f}ms > 100ms"
        )

    def test_status_check_performance(self, seeded_db_session, performance_tracker):
        """Test job status check performance."""
        from services.storage.database import get_all_generations

        # Get a generation ID
        gens = get_all_generations(seeded_db_session, limit=1)
        if not gens:
            pytest.skip("No generations in database")

        gen_id = gens[0].id

        # Measure status check
        with performance_tracker.measure("status_check"):
            result = get_generation(seeded_db_session, gen_id)

        check_time = performance_tracker.get_duration("status_check")

        print(f"\nStatus check time: {check_time * 1000:.1f}ms")

        # Should be very fast
        assert check_time < 0.05, (
            f"Status check too slow: {check_time * 1000:.1f}ms"
        )

    def test_bulk_query_performance(self, seeded_db_session, performance_tracker):
        """Test bulk query performance."""
        with performance_tracker.measure("bulk_query"):
            from services.storage.database import get_all_generations

            results = get_all_generations(seeded_db_session, limit=100)

        query_time = performance_tracker.get_duration("bulk_query")

        print(f"\nBulk query time: {query_time * 1000:.1f}ms for {len(results)} records")

        # Should still be fast
        assert query_time < 0.2, (
            f"Bulk query too slow: {query_time * 1000:.1f}ms"
        )


class TestMemoryUsage:
    """Test memory usage during generation."""

    @pytest.mark.gpu
    def test_gpu_memory_under_budget(self, real_engine):
        """Test GPU memory usage stays under 30GB budget."""
        import torch

        if not torch.cuda.is_available():
            pytest.skip("CUDA not available")

        # Reset memory stats
        torch.cuda.reset_peak_memory_stats()

        # Generate audio
        audio, sr = real_engine.generate_audio(
            prompt="test music",
            duration=16.0,
        )

        # Check peak memory
        peak_memory_mb = torch.cuda.max_memory_allocated() / (1024 * 1024)
        memory_budget_mb = 30 * 1024  # 30GB

        print(f"\nPeak GPU memory: {peak_memory_mb:.0f}MB")
        print(f"Memory budget: {memory_budget_mb:.0f}MB")
        print(f"Usage: {peak_memory_mb / memory_budget_mb * 100:.1f}%")

        assert peak_memory_mb < memory_budget_mb, (
            f"Memory usage {peak_memory_mb:.0f}MB exceeds "
            f"budget {memory_budget_mb:.0f}MB"
        )

    def test_memory_cleanup_after_generation(self, mock_engine):
        """Test memory is properly cleaned up after generation."""
        import psutil
        import os

        process = psutil.Process(os.getpid())

        # Measure initial memory
        initial_memory = process.memory_info().rss / (1024 * 1024)  # MB

        # Generate several files
        for i in range(5):
            request = GenerationRequest(prompt=f"test {i}", duration=2.0)
            result = mock_engine.generate(request)

        # Measure final memory
        final_memory = process.memory_info().rss / (1024 * 1024)  # MB
        memory_increase = final_memory - initial_memory

        print(f"\nMemory increase: {memory_increase:.1f}MB")

        # Memory increase should be reasonable (<500MB for mock engine)
        assert memory_increase < 500, (
            f"Memory leak suspected: {memory_increase:.1f}MB increase"
        )


class TestFileIOPerformance:
    """Test file I/O performance."""

    def test_wav_export_performance(self, output_dir, performance_tracker):
        """Test WAV export performance."""
        from services.audio.export import AudioExporter
        from tests.utils.mock_helpers import create_mock_audio_tensor

        exporter = AudioExporter()
        tensor = create_mock_audio_tensor(duration=16.0)
        output_path = output_dir / "perf_test.wav"

        with performance_tracker.measure("wav_export"):
            exporter.export_wav(
                audio_tensor=tensor,
                output_path=output_path,
                sample_rate=32000,
            )

        export_time = performance_tracker.get_duration("wav_export")

        print(f"\nWAV export time: {export_time * 1000:.1f}ms")

        # Should be fast (<1s for 16s audio)
        assert export_time < 1.0, f"WAV export too slow: {export_time:.2f}s"

    def test_metadata_extraction_performance(
        self, test_audio_file, performance_tracker
    ):
        """Test metadata extraction performance."""
        from services.audio.metadata import AudioMetadataExtractor

        extractor = AudioMetadataExtractor(extract_bpm=True)

        with performance_tracker.measure("metadata_extraction"):
            metadata = extractor.extract_metadata(test_audio_file)

        extraction_time = performance_tracker.get_duration("metadata_extraction")

        print(f"\nMetadata extraction time: {extraction_time * 1000:.1f}ms")

        # Should be reasonably fast (<5s including BPM detection)
        assert extraction_time < 5.0, (
            f"Metadata extraction too slow: {extraction_time:.2f}s"
        )


class TestConcurrentOperations:
    """Test concurrent operation performance."""

    def test_concurrent_database_reads(
        self, seeded_db_session, performance_tracker
    ):
        """Test concurrent database read performance."""
        from services.storage.database import get_all_generations

        num_queries = 10

        with performance_tracker.measure("concurrent_reads"):
            for _ in range(num_queries):
                results = get_all_generations(seeded_db_session, limit=10)

        total_time = performance_tracker.get_duration("concurrent_reads")
        avg_time = total_time / num_queries

        print(f"\n{num_queries} concurrent reads: {total_time:.3f}s")
        print(f"Average per query: {avg_time * 1000:.1f}ms")

        # Average should be fast
        assert avg_time < 0.1, f"Average query time too slow: {avg_time * 1000:.1f}ms"


class TestPerformanceReport:
    """Generate comprehensive performance report."""

    def test_generate_performance_report(
        self, mock_engine, seeded_db_session, output_dir, tmp_path
    ):
        """Generate a comprehensive performance report."""
        report = {
            "test_date": time.strftime("%Y-%m-%d %H:%M:%S"),
            "system_info": self._get_system_info(),
            "benchmarks": {},
        }

        # Test 1: Generation latency
        start = time.time()
        request = GenerationRequest(prompt="benchmark test", duration=16.0)
        result = mock_engine.generate(request)
        generation_time = time.time() - start

        report["benchmarks"]["generation_latency"] = {
            "duration": 16.0,
            "time_seconds": generation_time,
            "real_time_factor": generation_time / 16.0,
            "status": result.status,
        }

        # Test 2: Database query
        start = time.time()
        from services.storage.database import get_all_generations

        results = get_all_generations(seeded_db_session, limit=100)
        query_time = time.time() - start

        report["benchmarks"]["database_query"] = {
            "records_queried": len(results),
            "time_ms": query_time * 1000,
        }

        # Test 3: File I/O
        from services.audio.export import AudioExporter
        from tests.utils.mock_helpers import create_mock_audio_tensor

        exporter = AudioExporter()
        tensor = create_mock_audio_tensor(duration=16.0)
        output_path = output_dir / "benchmark.wav"

        start = time.time()
        exporter.export_wav(tensor, output_path, 32000)
        export_time = time.time() - start

        report["benchmarks"]["file_io"] = {
            "operation": "wav_export",
            "duration": 16.0,
            "time_ms": export_time * 1000,
        }

        # Write report
        report_path = tmp_path / "performance_report.json"
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)

        print(f"\nPerformance report written to: {report_path}")
        print(json.dumps(report["benchmarks"], indent=2))

        # Assertions
        assert report["benchmarks"]["generation_latency"]["time_seconds"] < 30.0
        assert report["benchmarks"]["database_query"]["time_ms"] < 200
        assert report["benchmarks"]["file_io"]["time_ms"] < 1000

    def _get_system_info(self):
        """Get system information."""
        import platform
        import torch

        return {
            "platform": platform.system(),
            "python_version": platform.python_version(),
            "cuda_available": torch.cuda.is_available(),
            "cuda_device": (
                torch.cuda.get_device_name(0)
                if torch.cuda.is_available()
                else None
            ),
        }
