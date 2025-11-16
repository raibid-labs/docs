# DGX Music - Development Automation
# Task automation for AI music generation platform

# Default recipe - show available commands
default:
    @just --list

# === Environment Setup ===

# Initialize project (first-time setup)
init:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🎵 Initializing DGX Music project..."

    # Create virtual environment
    if [ ! -d "venv" ]; then
        echo "Creating Python virtual environment..."
        python3 -m venv venv
    fi

    # Install dependencies
    source venv/bin/activate
    pip install --upgrade pip
    pip install -r requirements.txt || echo "requirements.txt not found - will create later"

    # Create directory structure
    mkdir -p services/{orchestrator,generation,rendering,integration}
    mkdir -p k8s/{base,dev,prod}
    mkdir -p scripts/{nushell,bash}
    mkdir -p configs
    mkdir -p data/{models,outputs,logs}
    mkdir -p tests/{unit,integration}

    echo "✅ Project initialized!"
    echo "Next steps:"
    echo "  1. Activate venv: source venv/bin/activate"
    echo "  2. Validate GPU: just validate-gpu"
    echo "  3. Install models: just install-models"

# Validate DGX Spark GPU/CUDA availability (CRITICAL for MVP)
validate-gpu:
    #!/usr/bin/env python3
    import sys
    try:
        import torch
        print("🔍 Checking CUDA availability...")
        if torch.cuda.is_available():
            print(f"✅ CUDA is available!")
            print(f"   GPU: {torch.cuda.get_device_name(0)}")
            print(f"   CUDA Version: {torch.version.cuda}")
            print(f"   Total Memory: {torch.cuda.get_device_properties(0).total_memory / 1e9:.2f} GB")
            sys.exit(0)
        else:
            print("⚠️  CUDA is NOT available")
            print("   This is a BLOCKER for MVP")
            print("   Options:")
            print("   1. CPU fallback (5-10x slower)")
            print("   2. Cloud GPU hybrid")
            print("   3. Alternative model (Stable Audio Open Small)")
            sys.exit(1)
    except ImportError:
        print("❌ PyTorch not installed")
        print("   Run: pip install torch torchvision torchaudio")
        sys.exit(1)

# === Model Management ===

# Download and install AI models
install-models:
    #!/usr/bin/env bash
    set -euo pipefail
    source venv/bin/activate

    echo "📥 Downloading AI models..."
    mkdir -p data/models

    # MusicGen Small (primary MVP model)
    python3 -c "
    from audiocraft.models import MusicGen
    print('Downloading MusicGen Small (~8GB)...')
    model = MusicGen.get_pretrained('small')
    print('✅ MusicGen Small installed')
    "

    echo "✅ Models installed to data/models/"

# Test model inference
test-model:
    #!/usr/bin/env bash
    set -euo pipefail
    source venv/bin/activate

    python3 -c "
    import time
    from audiocraft.models import MusicGen

    print('🧪 Testing MusicGen inference...')
    model = MusicGen.get_pretrained('small')
    model.set_generation_params(duration=8)

    start = time.time()
    wav = model.generate(['test generation hip hop beat'])
    elapsed = time.time() - start

    print(f'✅ Generation complete in {elapsed:.1f}s')
    if elapsed < 30:
        print('   Performance: EXCELLENT (<30s target met)')
    elif elapsed < 60:
        print('   Performance: ACCEPTABLE (<60s)')
    else:
        print('   ⚠️  Performance: SLOW (>60s - consider alternatives)')
    "

# === Development Commands ===

# Run the generation service (API)
serve PORT="8000":
    #!/usr/bin/env bash
    source venv/bin/activate
    echo "🚀 Starting DGX Music API on port {{PORT}}..."
    uvicorn services.generation.api:app --host 0.0.0.0 --port {{PORT}} --reload

# Generate music from CLI
generate PROMPT DURATION="16":
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -m services.generation.cli generate "{{PROMPT}}" --duration {{DURATION}}

# List generation history
history:
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -m services.generation.cli history

# === Testing ===

# Run all tests
test:
    #!/usr/bin/env bash
    source venv/bin/activate
    pytest tests/ -v

# Run tests with coverage
test-coverage:
    #!/usr/bin/env bash
    source venv/bin/activate
    pytest tests/ --cov=services --cov-report=html --cov-report=term

# Run unit tests only
test-unit:
    #!/usr/bin/env bash
    source venv/bin/activate
    pytest tests/unit/ -v

# Run integration tests
test-integration:
    #!/usr/bin/env bash
    source venv/bin/activate
    pytest tests/integration/ -v

# === Code Quality ===

# Format code with ruff
format:
    #!/usr/bin/env bash
    source venv/bin/activate
    ruff format services/ tests/

# Lint code
lint:
    #!/usr/bin/env bash
    source venv/bin/activate
    ruff check services/ tests/

# Type check with mypy
typecheck:
    #!/usr/bin/env bash
    source venv/bin/activate
    mypy services/ --ignore-missing-imports

# Run all quality checks
quality: lint typecheck
    @echo "✅ Code quality checks complete"

# === Database ===

# Initialize SQLite database
db-init:
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -c "
    from services.storage.database import init_db
    print('🗄️  Initializing database...')
    init_db()
    print('✅ Database initialized: data/generations.db')
    "

# Run database migrations
db-migrate:
    #!/usr/bin/env bash
    source venv/bin/activate
    alembic upgrade head

# Reset database (WARNING: deletes all data)
db-reset:
    #!/usr/bin/env bash
    echo "⚠️  This will delete all generation history. Continue? (y/n)"
    read -r response
    if [ "$response" = "y" ]; then
        rm -f data/generations.db
        just db-init
        echo "✅ Database reset complete"
    else
        echo "Cancelled"
    fi

# === Deployment ===

# Deploy to DGX Spark (systemd)
deploy-dgx:
    @./scripts/bash/deploy-dgx.sh

# Build Docker image for Kubernetes
docker-build TAG="latest":
    docker build -t dgx-music:{{TAG}} .

# Start Kubernetes development environment
k8s-dev:
    tilt up

# === Orchestrator ===

# Launch orchestrator agent to process GitHub issues
orchestrate:
    @./scripts/nushell/launch-orchestrator.nu

# Check orchestrator status
orchestrator-status:
    @./scripts/nushell/check-status.nu

# === Audio Utilities ===

# Export generation to Ardour template
export-ardour JOB_ID:
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -m services.integration.ardour export {{JOB_ID}}

# Normalize audio loudness
normalize-audio INPUT OUTPUT:
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -c "
    from services.audio.processing import normalize_loudness
    normalize_loudness('{{INPUT}}', '{{OUTPUT}}', target_lufs=-16.0)
    "

# === Monitoring ===

# View API logs
logs:
    journalctl -u dgx-music -f

# Check service health
health:
    curl -f http://localhost:8000/health || echo "Service not running"

# Monitor GPU usage
gpu-status:
    nvidia-smi --query-gpu=name,memory.used,memory.total,temperature.gpu,utilization.gpu --format=csv

# === Cleanup ===

# Clean generated files and caches
clean:
    #!/usr/bin/env bash
    echo "🧹 Cleaning..."
    rm -rf data/outputs/*.wav
    rm -rf data/logs/*.log
    rm -rf __pycache__ **/__pycache__
    rm -rf .pytest_cache
    rm -rf .mypy_cache
    rm -rf .ruff_cache
    echo "✅ Cleanup complete"

# Deep clean (includes venv and models)
clean-all:
    #!/usr/bin/env bash
    echo "⚠️  This will delete venv and models. Continue? (y/n)"
    read -r response
    if [ "$response" = "y" ]; then
        just clean
        rm -rf venv/
        rm -rf data/models/
        echo "✅ Deep cleanup complete"
    else
        echo "Cancelled"
    fi

# === Documentation ===

# Generate API documentation
docs-api:
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -c "
    from services.generation.api import app
    import json
    spec = app.openapi()
    with open('docs/api-spec.json', 'w') as f:
        json.dump(spec, f, indent=2)
    print('✅ API spec written to docs/api-spec.json')
    "

# Serve documentation locally
docs-serve:
    @echo "📚 Serving documentation at http://localhost:8080"
    python3 -m http.server 8080 --directory docs/

# === Performance Benchmarking ===

# Run performance benchmarks
benchmark:
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -m tests.benchmarks.generation_performance

# Profile memory usage
profile-memory:
    #!/usr/bin/env bash
    source venv/bin/activate
    python3 -m memory_profiler services/generation/engine.py

# === CI/CD ===

# Run CI checks (used in GitHub Actions)
ci: quality test
    @echo "✅ All CI checks passed"

# Prepare release
release VERSION:
    #!/usr/bin/env bash
    echo "📦 Preparing release {{VERSION}}..."

    # Update version
    sed -i 's/__version__ = ".*"/__version__ = "{{VERSION}}"/' services/__init__.py

    # Create git tag
    git tag -a v{{VERSION}} -m "Release {{VERSION}}"

    echo "✅ Release {{VERSION}} prepared"
    echo "   Push with: git push origin v{{VERSION}}"
