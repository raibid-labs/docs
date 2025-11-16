#!/usr/bin/env python3
"""
Performance Benchmark Script for Music Generation
=================================================

This script benchmarks the MusicGen model on DGX Spark to validate
Week 1 performance targets:
- <30s for 16s audio (target)
- <60s for 16s audio (acceptable)
- >60s for 16s audio (BLOCKER - requires mitigation)

Outputs a comprehensive report with recommendations.
"""

import sys
import time
from pathlib import Path
from typing import List, Dict, Any

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from services.generation.engine import MusicGenerationEngine
from services.generation.models import PerformanceBenchmark


class BenchmarkRunner:
    """Runs comprehensive performance benchmarks."""

    def __init__(self):
        self.engine = None
        self.results: List[PerformanceBenchmark] = []

    def setup(self):
        """Initialize the engine."""
        print("="*80)
        print("DGX MUSIC - PERFORMANCE BENCHMARK")
        print("="*80)
        print("\nInitializing engine...")

        try:
            self.engine = MusicGenerationEngine(
                model_name="small",
                use_gpu=True,
                enable_caching=True,
            )
            print("✅ Engine initialized")

            # Load model
            print("Loading MusicGen Small model...")
            start = time.time()
            self.engine.load_model()
            load_time = time.time() - start
            print(f"✅ Model loaded in {load_time:.2f}s")

        except Exception as e:
            print(f"❌ Failed to initialize: {e}")
            raise

    def run_single_benchmark(
        self,
        duration: float,
        prompt: str = "electronic dance music with synth melody",
    ) -> PerformanceBenchmark:
        """Run a single benchmark."""
        print(f"\n{'─'*80}")
        print(f"Benchmark: {duration}s audio")
        print(f"Prompt: {prompt}")
        print(f"{'─'*80}")

        try:
            benchmark = self.engine.benchmark(duration=duration)
            self.results.append(benchmark)

            print(f"✅ Generation Time: {benchmark.generation_time:.2f}s")
            print(f"   Real-Time Factor: {benchmark.real_time_factor:.2f}x")
            print(f"   Peak Memory: {benchmark.peak_memory_mb:.1f}MB")

            return benchmark

        except Exception as e:
            print(f"❌ Benchmark failed: {e}")
            raise

    def run_suite(self):
        """Run complete benchmark suite."""
        print("\n" + "="*80)
        print("RUNNING BENCHMARK SUITE")
        print("="*80)

        # Test different durations
        durations = [4.0, 8.0, 16.0, 30.0]

        for duration in durations:
            try:
                self.run_single_benchmark(duration)
                # Small delay between benchmarks
                time.sleep(1)
            except Exception as e:
                print(f"⚠️  Benchmark for {duration}s failed: {e}")

    def analyze_results(self) -> Dict[str, Any]:
        """Analyze benchmark results and generate report."""
        if not self.results:
            return {"status": "no_results", "message": "No benchmarks completed"}

        # Focus on 16s benchmark (MVP target)
        target_benchmark = next(
            (b for b in self.results if b.duration == 16.0),
            None
        )

        if not target_benchmark:
            target_benchmark = self.results[0]

        analysis = {
            "target_duration": 16.0,
            "generation_time": target_benchmark.generation_time,
            "real_time_factor": target_benchmark.real_time_factor,
            "peak_memory_mb": target_benchmark.peak_memory_mb,
            "cuda_available": target_benchmark.cuda_available,
            "model_name": target_benchmark.model_name,
        }

        # Performance assessment
        gen_time = target_benchmark.generation_time

        if gen_time < 30.0:
            analysis["status"] = "excellent"
            analysis["assessment"] = "EXCELLENT - Meets <30s target"
            analysis["recommendation"] = "Proceed with MVP implementation"
        elif gen_time < 60.0:
            analysis["status"] = "acceptable"
            analysis["assessment"] = "ACCEPTABLE - Within <60s threshold"
            analysis["recommendation"] = "Proceed with MVP, consider optimization in Phase 2"
        else:
            analysis["status"] = "blocker"
            analysis["assessment"] = "BLOCKER - Exceeds 60s threshold"
            analysis["recommendation"] = "CRITICAL: Implement mitigation strategy"

        # Memory assessment
        memory_mb = target_benchmark.peak_memory_mb
        memory_budget_mb = 30 * 1024  # 30GB

        if memory_mb < memory_budget_mb:
            analysis["memory_status"] = "within_budget"
            analysis["memory_assessment"] = f"Within {memory_budget_mb/1024:.0f}GB budget"
        else:
            analysis["memory_status"] = "over_budget"
            analysis["memory_assessment"] = f"EXCEEDS {memory_budget_mb/1024:.0f}GB budget"

        return analysis

    def print_report(self):
        """Print comprehensive benchmark report."""
        analysis = self.analyze_results()

        print("\n" + "="*80)
        print("BENCHMARK REPORT")
        print("="*80)

        print("\n📊 RESULTS SUMMARY:")
        print(f"   Model: {analysis.get('model_name', 'unknown')}")
        print(f"   CUDA Available: {analysis.get('cuda_available', False)}")
        print(f"   Target Duration: {analysis.get('target_duration', 0)}s")
        print(f"   Generation Time: {analysis.get('generation_time', 0):.2f}s")
        print(f"   Real-Time Factor: {analysis.get('real_time_factor', 0):.2f}x")
        print(f"   Peak Memory: {analysis.get('peak_memory_mb', 0):.1f}MB")

        print(f"\n🎯 PERFORMANCE ASSESSMENT:")
        status = analysis.get('status', 'unknown')
        assessment = analysis.get('assessment', 'Unknown')

        if status == "excellent":
            print(f"   ✅ {assessment}")
        elif status == "acceptable":
            print(f"   ⚠️  {assessment}")
        else:
            print(f"   🚨 {assessment}")

        print(f"\n💾 MEMORY ASSESSMENT:")
        memory_status = analysis.get('memory_status', 'unknown')
        memory_assessment = analysis.get('memory_assessment', 'Unknown')

        if memory_status == "within_budget":
            print(f"   ✅ {memory_assessment}")
        else:
            print(f"   🚨 {memory_assessment}")

        print(f"\n📋 RECOMMENDATION:")
        recommendation = analysis.get('recommendation', 'Unknown')
        print(f"   {recommendation}")

        # Detailed results table
        print("\n📈 DETAILED RESULTS:")
        print(f"{'Duration':<12} {'Gen Time':<12} {'RTF':<10} {'Memory':<15}")
        print("─" * 80)

        for result in self.results:
            print(
                f"{result.duration:<12.1f} "
                f"{result.generation_time:<12.2f} "
                f"{result.real_time_factor:<10.2f} "
                f"{result.peak_memory_mb:<15.1f}"
            )

        # Mitigation strategies if needed
        if status == "blocker":
            self.print_mitigation_strategies()

        print("\n" + "="*80)

    def print_mitigation_strategies(self):
        """Print mitigation strategies for performance issues."""
        print("\n🚨 MITIGATION STRATEGIES REQUIRED:")
        print("\n   Option 1: CPU Fallback")
        print("   ├─ Use PyTorch CPU-only mode")
        print("   ├─ Expected: 5-10x slower than current")
        print("   ├─ Acceptable for: Demo/validation only")
        print("   └─ Action: Set use_gpu=False in config")

        print("\n   Option 2: Cloud GPU Hybrid (RECOMMENDED)")
        print("   ├─ DGX Spark: API/storage/orchestration")
        print("   ├─ Cloud GPU: Generation only")
        print("   ├─ Adds: ~500ms latency")
        print("   ├─ Cost: ~$0.50-1.00/hr")
        print("   └─ Action: Implement remote generation API")

        print("\n   Option 3: Smaller Model")
        print("   ├─ Switch to: MusicGen Tiny")
        print("   ├─ Trade-off: Lower quality")
        print("   ├─ Expected: 2-3x faster")
        print("   └─ Action: Change model_name to 'tiny'")

        print("\n   Option 4: Alternative Model")
        print("   ├─ Switch to: Stable Audio Open Small")
        print("   ├─ ARM64-optimized")
        print("   ├─ May have better performance")
        print("   └─ Action: Evaluate Stable Audio")

    def save_results(self, output_file: str = "benchmark_results.txt"):
        """Save results to file."""
        analysis = self.analyze_results()

        output_path = Path(output_file)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, 'w') as f:
            f.write("DGX MUSIC - BENCHMARK RESULTS\n")
            f.write("="*80 + "\n\n")

            for key, value in analysis.items():
                f.write(f"{key}: {value}\n")

            f.write("\n\nDETAILED RESULTS:\n")
            f.write("─"*80 + "\n")

            for result in self.results:
                f.write(f"Duration: {result.duration}s\n")
                f.write(f"Generation Time: {result.generation_time:.2f}s\n")
                f.write(f"RTF: {result.real_time_factor:.2f}x\n")
                f.write(f"Memory: {result.peak_memory_mb:.1f}MB\n")
                f.write("─"*80 + "\n")

        print(f"\n💾 Results saved to: {output_path}")


def main():
    """Main benchmark execution."""
    runner = BenchmarkRunner()

    try:
        # Setup
        runner.setup()

        # Run benchmarks
        runner.run_suite()

        # Print report
        runner.print_report()

        # Save results
        runner.save_results("data/logs/benchmark_results.txt")

        # Determine exit code based on results
        analysis = runner.analyze_results()
        status = analysis.get('status', 'unknown')

        if status == "excellent":
            print("\n✅ All performance targets met - ready for MVP")
            sys.exit(0)
        elif status == "acceptable":
            print("\n⚠️  Performance acceptable - proceed with caution")
            sys.exit(0)
        else:
            print("\n🚨 Performance BLOCKER - mitigation required")
            sys.exit(1)

    except Exception as e:
        print(f"\n❌ Benchmark failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
