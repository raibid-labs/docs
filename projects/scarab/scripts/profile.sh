#!/bin/bash
# Performance profiling script for Scarab

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_color() {
    color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Parse command line arguments
PROFILE_TYPE="${1:-all}"
PROFILE_DURATION="${2:-30}"
OUTPUT_DIR="${3:-./profiling-results}"

# Create output directory
mkdir -p "$OUTPUT_DIR"
print_color "$GREEN" "Output directory: $OUTPUT_DIR"

# Function to run CPU profiling with perf
profile_cpu() {
    print_color "$BLUE" "\n=== CPU Profiling with perf ==="

    if ! command_exists perf; then
        print_color "$RED" "perf not found. Install with: apt-get install linux-tools-generic"
        return 1
    fi

    # Build with debug symbols
    print_color "$YELLOW" "Building with debug symbols..."
    cargo build --profile=profiling --features=profiling

    # Record performance data
    print_color "$YELLOW" "Recording CPU profile for ${PROFILE_DURATION}s..."
    sudo perf record -F 99 -a -g -- timeout ${PROFILE_DURATION}s target/profiling/scarab-daemon &
    PERF_PID=$!

    # Start the client to generate load
    sleep 2
    timeout $((PROFILE_DURATION - 2))s target/profiling/scarab-client &

    wait $PERF_PID 2>/dev/null || true

    # Generate flame graph
    if command_exists flamegraph; then
        print_color "$YELLOW" "Generating flame graph..."
        sudo perf script | flamegraph > "$OUTPUT_DIR/flamegraph.svg"
        print_color "$GREEN" "Flame graph saved to: $OUTPUT_DIR/flamegraph.svg"
    else
        print_color "$YELLOW" "flamegraph not found. Install with: cargo install flamegraph"

        # Generate perf report instead
        sudo perf report --stdio > "$OUTPUT_DIR/perf-report.txt"
        print_color "$GREEN" "Perf report saved to: $OUTPUT_DIR/perf-report.txt"
    fi

    # Clean up
    sudo rm -f perf.data perf.data.old
}

# Function to run memory profiling with Valgrind
profile_memory() {
    print_color "$BLUE" "\n=== Memory Profiling with Valgrind ==="

    if ! command_exists valgrind; then
        print_color "$RED" "valgrind not found. Install with: apt-get install valgrind"
        return 1
    fi

    # Build in release mode for memory profiling
    print_color "$YELLOW" "Building release binary..."
    cargo build --release

    # Run with Massif
    print_color "$YELLOW" "Running Massif memory profiler..."
    valgrind --tool=massif \
        --massif-out-file="$OUTPUT_DIR/massif.out" \
        --time-unit=B \
        --detailed-freq=1 \
        --max-snapshots=100 \
        --threshold=0.1 \
        timeout ${PROFILE_DURATION}s target/release/scarab-daemon &

    MASSIF_PID=$!

    # Generate some load
    sleep 2
    timeout $((PROFILE_DURATION - 2))s target/release/scarab-client &

    wait $MASSIF_PID 2>/dev/null || true

    # Generate report
    if command_exists ms_print; then
        ms_print "$OUTPUT_DIR/massif.out" > "$OUTPUT_DIR/massif-report.txt"
        print_color "$GREEN" "Massif report saved to: $OUTPUT_DIR/massif-report.txt"
    fi

    # Check for memory leaks
    print_color "$YELLOW" "Checking for memory leaks..."
    valgrind --leak-check=full \
        --show-leak-kinds=all \
        --track-origins=yes \
        --log-file="$OUTPUT_DIR/memcheck.log" \
        timeout 5s target/release/scarab-daemon &

    LEAK_PID=$!
    sleep 2
    timeout 3s target/release/scarab-client &
    wait $LEAK_PID 2>/dev/null || true

    print_color "$GREEN" "Memory leak report saved to: $OUTPUT_DIR/memcheck.log"
}

# Function to run Tracy profiling
profile_tracy() {
    print_color "$BLUE" "\n=== Tracy Profiling ==="

    # Build with Tracy support
    print_color "$YELLOW" "Building with Tracy support..."
    cargo build --release --features=tracy

    print_color "$YELLOW" "Starting Tracy-enabled daemon..."
    print_color "$YELLOW" "Please connect Tracy Profiler to capture the trace"

    # Run the daemon with Tracy
    timeout ${PROFILE_DURATION}s target/release/scarab-daemon &
    TRACY_PID=$!

    # Generate load
    sleep 2
    timeout $((PROFILE_DURATION - 2))s target/release/scarab-client &

    wait $TRACY_PID 2>/dev/null || true

    print_color "$GREEN" "Tracy profiling session completed"
    print_color "$YELLOW" "Save the trace from Tracy Profiler GUI"
}

# Function to run benchmarks
run_benchmarks() {
    print_color "$BLUE" "\n=== Running Criterion Benchmarks ==="

    # Run daemon benchmarks
    print_color "$YELLOW" "Running daemon benchmarks..."
    cargo bench --package scarab-daemon --no-fail-fast

    # Run client benchmarks
    print_color "$YELLOW" "Running client benchmarks..."
    cargo bench --package scarab-client --no-fail-fast

    # Copy benchmark results
    if [ -d "target/criterion" ]; then
        cp -r target/criterion "$OUTPUT_DIR/"
        print_color "$GREEN" "Benchmark results saved to: $OUTPUT_DIR/criterion/"
    fi
}

# Function to collect system metrics
collect_metrics() {
    print_color "$BLUE" "\n=== Collecting System Metrics ==="

    # Start daemon in background
    print_color "$YELLOW" "Starting daemon with metrics collection..."
    RUST_LOG=debug cargo run --release --features=profiling --bin scarab-daemon > "$OUTPUT_DIR/daemon.log" 2>&1 &
    DAEMON_PID=$!

    sleep 2

    # Start client
    cargo run --release --bin scarab-client > "$OUTPUT_DIR/client.log" 2>&1 &
    CLIENT_PID=$!

    # Collect metrics periodically
    print_color "$YELLOW" "Collecting metrics for ${PROFILE_DURATION}s..."

    for i in $(seq 1 $PROFILE_DURATION); do
        # CPU and memory usage
        ps aux | grep -E "(scarab-daemon|scarab-client)" | grep -v grep >> "$OUTPUT_DIR/process-stats.log"

        # System load
        uptime >> "$OUTPUT_DIR/system-load.log"

        # IO stats
        if command_exists iostat; then
            iostat -x 1 1 >> "$OUTPUT_DIR/io-stats.log"
        fi

        sleep 1
    done

    # Stop processes
    kill $CLIENT_PID 2>/dev/null || true
    kill $DAEMON_PID 2>/dev/null || true

    print_color "$GREEN" "Metrics collected in: $OUTPUT_DIR/"
}

# Main execution
print_color "$GREEN" "Scarab Performance Profiling Tool"
print_color "$GREEN" "================================="

case "$PROFILE_TYPE" in
    cpu)
        profile_cpu
        ;;
    memory)
        profile_memory
        ;;
    tracy)
        profile_tracy
        ;;
    bench)
        run_benchmarks
        ;;
    metrics)
        collect_metrics
        ;;
    all)
        profile_cpu
        profile_memory
        run_benchmarks
        collect_metrics
        ;;
    *)
        print_color "$RED" "Unknown profile type: $PROFILE_TYPE"
        print_color "$YELLOW" "Usage: $0 [cpu|memory|tracy|bench|metrics|all] [duration] [output_dir]"
        exit 1
        ;;
esac

print_color "$GREEN" "\n=== Profiling Complete ==="
print_color "$GREEN" "Results saved to: $OUTPUT_DIR/"

# Generate summary report
print_color "$YELLOW" "\nGenerating summary report..."
{
    echo "Scarab Performance Profile Summary"
    echo "=================================="
    echo "Date: $(date)"
    echo "Profile Type: $PROFILE_TYPE"
    echo "Duration: ${PROFILE_DURATION}s"
    echo ""

    if [ -f "$OUTPUT_DIR/perf-report.txt" ]; then
        echo "Top CPU Consumers:"
        head -20 "$OUTPUT_DIR/perf-report.txt" | grep -E "^\s+[0-9]+\.[0-9]+%"
        echo ""
    fi

    if [ -f "$OUTPUT_DIR/massif-report.txt" ]; then
        echo "Peak Memory Usage:"
        grep "peak" "$OUTPUT_DIR/massif-report.txt" | head -5
        echo ""
    fi

    if [ -f "$OUTPUT_DIR/memcheck.log" ]; then
        echo "Memory Leak Summary:"
        grep "LEAK SUMMARY" -A 5 "$OUTPUT_DIR/memcheck.log"
        echo ""
    fi

    if [ -d "$OUTPUT_DIR/criterion" ]; then
        echo "Benchmark Results:"
        find "$OUTPUT_DIR/criterion" -name "estimates.json" | while read f; do
            echo "  $(basename $(dirname $(dirname $f)))"
        done
        echo ""
    fi
} > "$OUTPUT_DIR/summary.txt"

print_color "$GREEN" "Summary report: $OUTPUT_DIR/summary.txt"