#!/usr/bin/env python3
"""
Quick test script to verify database initialization.

This script tests the database setup without requiring a full venv.
"""

import sys
from pathlib import Path

# Add project to path
sys.path.insert(0, str(Path(__file__).parent))

try:
    from services.storage import init_db, get_session, create_generation, get_database_stats
    from services.storage.schema import GenerationStatus

    print("Initializing database...")
    init_db()
    print("Database initialized successfully!")

    print("\nTesting database operations...")

    # Test creating a generation
    with get_session() as session:
        gen = create_generation(
            session=session,
            prompt="test hip hop beat at 140 BPM",
            model_name="musicgen-small",
            duration_seconds=16.0,
            sample_rate=32000,
            channels=2,
            file_path="outputs/test.wav",
            metadata={"bpm": 140, "genre": "hip hop"}
        )
        print(f"Created generation: {gen.id}")
        print(f"  Prompt: {gen.prompt}")
        print(f"  Status: {gen.status}")
        print(f"  Metadata: {gen.get_metadata()}")

    # Test database stats
    with get_session() as session:
        stats = get_database_stats(session)
        print("\nDatabase statistics:")
        for key, value in stats.items():
            print(f"  {key}: {value}")

    print("\nAll tests passed!")

except ImportError as e:
    print(f"Error: Missing dependencies - {e}")
    print("\nPlease install dependencies:")
    print("  pip install sqlalchemy")
    sys.exit(1)
except Exception as e:
    print(f"Error: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
