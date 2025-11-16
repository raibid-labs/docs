#!/usr/bin/env python3
"""
GPU/CUDA Validation Script for DGX Spark
=========================================

This script is CRITICAL for Week 1 MVP validation. It checks:
1. PyTorch installation
2. CUDA availability
3. GPU device information
4. Memory capacity
5. ARM64 compatibility

If this script fails, the MVP is BLOCKED and requires immediate mitigation
(CPU fallback, cloud hybrid, or alternative model).
"""

import sys
from typing import Dict, Any


def validate_pytorch() -> bool:
    """Check if PyTorch is installed."""
    try:
        import torch
        print(f"✅ PyTorch version: {torch.__version__}")
        return True
    except ImportError:
        print("❌ PyTorch is not installed")
        print("   Install with: pip install torch torchvision torchaudio")
        return False


def check_cuda_availability() -> tuple[bool, Dict[str, Any]]:
    """
    Check CUDA availability and gather GPU information.

    Returns:
        Tuple of (cuda_available, gpu_info_dict)
    """
    try:
        import torch

        cuda_available = torch.cuda.is_available()

        if not cuda_available:
            return False, {}

        # Gather GPU information
        gpu_info = {
            "device_count": torch.cuda.device_count(),
            "current_device": torch.cuda.current_device(),
            "device_name": torch.cuda.get_device_name(0),
            "cuda_version": torch.version.cuda,
            "cudnn_version": torch.backends.cudnn.version(),
            "total_memory_gb": torch.cuda.get_device_properties(0).total_memory / 1e9,
            "major": torch.cuda.get_device_properties(0).major,
            "minor": torch.cuda.get_device_properties(0).minor,
        }

        return True, gpu_info

    except Exception as e:
        print(f"❌ Error checking CUDA: {e}")
        return False, {}


def check_arm64_compatibility():
    """Check if running on ARM64 architecture."""
    import platform

    machine = platform.machine()
    is_arm64 = machine in ["aarch64", "arm64"]

    print(f"🔍 System Architecture: {machine}")
    if is_arm64:
        print("✅ Running on ARM64 (DGX Spark compatible)")
    else:
        print(f"⚠️  Running on {machine} (not ARM64)")
        print("   Note: This is acceptable for development, but deployment requires ARM64")

    return is_arm64


def test_basic_tensor_ops() -> bool:
    """Test basic tensor operations on GPU."""
    try:
        import torch

        if not torch.cuda.is_available():
            return False

        # Create a small tensor and move to GPU
        x = torch.randn(100, 100).cuda()
        y = torch.randn(100, 100).cuda()
        z = torch.matmul(x, y)

        # Verify result is on GPU
        assert z.is_cuda, "Tensor operation failed to stay on GPU"

        print("✅ Basic GPU tensor operations working")
        return True

    except Exception as e:
        print(f"❌ GPU tensor operations failed: {e}")
        return False


def print_recommendations(cuda_available: bool, gpu_info: Dict[str, Any]):
    """Print recommendations based on validation results."""
    print("\n" + "="*60)
    print("VALIDATION SUMMARY")
    print("="*60)

    if cuda_available:
        print("\n✅ GPU VALIDATION PASSED")
        print(f"   Device: {gpu_info['device_name']}")
        print(f"   Memory: {gpu_info['total_memory_gb']:.2f} GB")
        print(f"   CUDA: {gpu_info['cuda_version']}")
        print("\n🎯 RECOMMENDATION: Proceed with MusicGen Small (8GB VRAM)")
        print("   Next steps:")
        print("   1. Run: just install-models")
        print("   2. Run: just test-model")
        print("   3. Benchmark generation performance")

    else:
        print("\n⚠️  GPU VALIDATION FAILED - MVP BLOCKED")
        print("\n🚨 CRITICAL MITIGATION REQUIRED:")
        print("\n   Option 1: CPU Fallback (Simple)")
        print("   - Use PyTorch CPU-only mode")
        print("   - Expected performance: 5-10x slower (60-300s per generation)")
        print("   - Acceptable for MVP validation only")
        print("   - Command: pip install torch torchvision torchaudio")

        print("\n   Option 2: Cloud GPU Hybrid (Recommended)")
        print("   - DGX Spark handles API/storage/orchestration")
        print("   - Remote GPU instances (AWS/GCP) handle generation")
        print("   - Adds ~500ms latency but reliable performance")
        print("   - Cost: ~$0.50-1.00 per GPU hour")

        print("\n   Option 3: Alternative Model")
        print("   - Switch to Stable Audio Open Small")
        print("   - Explicitly ARM64-optimized")
        print("   - Lower quality but proven compatibility")

        print("\n   Decision Matrix:")
        print("   - If budget allows: Option 2 (Cloud Hybrid)")
        print("   - If demo-only: Option 1 (CPU Fallback)")
        print("   - If quality flexible: Option 3 (Alternative Model)")


def main():
    """Main validation flow."""
    print("="*60)
    print("DGX MUSIC - GPU/CUDA VALIDATION")
    print("="*60)
    print("\nThis validation is CRITICAL for MVP - Week 1 Day 1")
    print("\n")

    # Step 1: Check PyTorch
    print("Step 1: Checking PyTorch installation...")
    if not validate_pytorch():
        sys.exit(1)

    # Step 2: Check ARM64
    print("\nStep 2: Checking system architecture...")
    check_arm64_compatibility()

    # Step 3: Check CUDA
    print("\nStep 3: Checking CUDA availability...")
    cuda_available, gpu_info = check_cuda_availability()

    if cuda_available:
        print("✅ CUDA is AVAILABLE!")
        print(f"   GPU Count: {gpu_info['device_count']}")
        print(f"   GPU Name: {gpu_info['device_name']}")
        print(f"   CUDA Version: {gpu_info['cuda_version']}")
        print(f"   cuDNN Version: {gpu_info['cudnn_version']}")
        print(f"   Total Memory: {gpu_info['total_memory_gb']:.2f} GB")
        print(f"   Compute Capability: {gpu_info['major']}.{gpu_info['minor']}")

        # Step 4: Test tensor operations
        print("\nStep 4: Testing GPU tensor operations...")
        test_basic_tensor_ops()

    else:
        print("❌ CUDA is NOT AVAILABLE")
        print("   This is a BLOCKER for the GPU-based MVP approach")

    # Print recommendations
    print_recommendations(cuda_available, gpu_info)

    # Exit code
    if cuda_available:
        print("\n✅ All validations passed - ready to proceed")
        sys.exit(0)
    else:
        print("\n⚠️  Validation failed - mitigation required before proceeding")
        sys.exit(1)


if __name__ == "__main__":
    main()
