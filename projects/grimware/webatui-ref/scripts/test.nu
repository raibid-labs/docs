#!/usr/bin/env nu

# test.nu - Comprehensive test runner for webatui project
#
# Usage:
#   nu scripts/test.nu [OPTIONS]
#
# Options:
#   --all             Run all tests (unit, integration, doc)
#   --unit            Run unit tests only
#   --integration     Run integration tests only
#   --doc             Run documentation tests only
#   --wasm            Run WASM-specific tests
#   --bench           Run benchmarks
#   --coverage        Generate coverage report
#   --filter PATTERN  Run tests matching pattern
#   --nocapture       Show test output
#   --release         Run tests in release mode
#   --help            Show this help message

def main [
    --all             # Run all tests
    --unit            # Run unit tests
    --integration     # Run integration tests
    --doc             # Run doc tests
    --wasm            # Run WASM tests
    --bench           # Run benchmarks
    --coverage        # Generate coverage
    --filter: string  # Filter tests by pattern
    --nocapture       # Show test output
    --release         # Run in release mode
    --help            # Show help
] {
    if $help {
        show_help
        return
    }

    print_header "🧪 Test Runner"

    # Determine which tests to run
    let run_all = $all or (not $unit and not $integration and not $doc and not $wasm and not $bench)

    if $coverage {
        run_with_coverage $filter $nocapture
        return
    }

    if $bench {
        run_benchmarks $filter
        return
    }

    if $run_all {
        run_all_tests $filter $nocapture $release
    } else {
        if $unit {
            run_unit_tests $filter $nocapture $release
        }
        if $integration {
            run_integration_tests $filter $nocapture $release
        }
        if $doc {
            run_doc_tests $filter $release
        }
        if $wasm {
            run_wasm_tests $filter $nocapture
        }
    }

    print_success "All tests completed!"
}

# Run all tests
def run_all_tests [filter: string, nocapture: bool, release: bool] {
    print_step "Running all tests"

    run_unit_tests $filter $nocapture $release
    run_integration_tests $filter $nocapture $release
    run_doc_tests $filter $release
}

# Run unit tests
def run_unit_tests [filter: string, nocapture: bool, release: bool] {
    print_step "Running unit tests"

    let mut cmd = ["cargo", "test", "--lib"]

    if $release {
        $cmd = ($cmd | append "--release")
    }

    if $filter != null {
        $cmd = ($cmd | append $filter)
    }

    $cmd = ($cmd | append "--")

    if $nocapture {
        $cmd = ($cmd | append "--nocapture")
    }

    $cmd = ($cmd | append "--test-threads=1")

    print_command $cmd

    let result = (do { run-external $cmd.0 ...$cmd.1.. } | complete)

    if $result.exit_code == 0 {
        print_success "Unit tests passed!"
    } else {
        print_error "Unit tests failed!"
        print $result.stderr
        exit 1
    }
}

# Run integration tests
def run_integration_tests [filter: string, nocapture: bool, release: bool] {
    print_step "Running integration tests"

    let mut cmd = ["cargo", "test", "--test", "*"]

    if $release {
        $cmd = ($cmd | append "--release")
    }

    if $filter != null {
        $cmd = ($cmd | append $filter)
    }

    $cmd = ($cmd | append "--")

    if $nocapture {
        $cmd = ($cmd | append "--nocapture")
    }

    print_command $cmd

    let result = (do { run-external $cmd.0 ...$cmd.1.. } | complete)

    if $result.exit_code == 0 {
        print_success "Integration tests passed!"
    } else {
        print_error "Integration tests failed!"
        print $result.stderr
        exit 1
    }
}

# Run documentation tests
def run_doc_tests [filter: string, release: bool] {
    print_step "Running documentation tests"

    let mut cmd = ["cargo", "test", "--doc"]

    if $release {
        $cmd = ($cmd | append "--release")
    }

    if $filter != null {
        $cmd = ($cmd | append $filter)
    }

    print_command $cmd

    let result = (do { run-external $cmd.0 ...$cmd.1.. } | complete)

    if $result.exit_code == 0 {
        print_success "Doc tests passed!"
    } else {
        print_error "Doc tests failed!"
        print $result.stderr
        exit 1
    }
}

# Run WASM tests
def run_wasm_tests [filter: string, nocapture: bool] {
    print_step "Running WASM tests"

    # Check if wasm-pack is installed
    let wasm_pack_installed = (which wasm-pack | length) > 0

    if not $wasm_pack_installed {
        print_error "wasm-pack is not installed!"
        print_info "Install with: cargo install wasm-pack"
        exit 1
    }

    let mut cmd = ["wasm-pack", "test", "--headless", "--firefox"]

    if $filter != null {
        $cmd = ($cmd | append ["--", $filter])
    }

    print_command $cmd

    let result = (do { run-external $cmd.0 ...$cmd.1.. } | complete)

    if $result.exit_code == 0 {
        print_success "WASM tests passed!"
    } else {
        print_error "WASM tests failed!"
        print $result.stderr
        exit 1
    }
}

# Run benchmarks
def run_benchmarks [filter: string] {
    print_step "Running benchmarks"

    let mut cmd = ["cargo", "bench"]

    if $filter != null {
        $cmd = ($cmd | append $filter)
    }

    print_command $cmd

    let result = (do { run-external $cmd.0 ...$cmd.1.. } | complete)

    if $result.exit_code == 0 {
        print_success "Benchmarks completed!"
    } else {
        print_error "Benchmarks failed!"
        print $result.stderr
        exit 1
    }
}

# Run tests with coverage
def run_with_coverage [filter: string, nocapture: bool] {
    print_step "Running tests with coverage"

    # Check if tarpaulin is installed
    let tarpaulin_installed = (which cargo-tarpaulin | length) > 0

    if not $tarpaulin_installed {
        print_error "cargo-tarpaulin is not installed!"
        print_info "Install with: cargo install cargo-tarpaulin"
        exit 1
    }

    let mut cmd = ["cargo", "tarpaulin", "--out", "Html", "--output-dir", "coverage"]

    if $filter != null {
        $cmd = ($cmd | append ["--", $filter])
    }

    if $nocapture {
        $cmd = ($cmd | append "--verbose")
    }

    print_command $cmd

    let result = (do { run-external $cmd.0 ...$cmd.1.. } | complete)

    if $result.exit_code == 0 {
        print_success "Coverage report generated!"
        print_info "Open coverage/index.html to view the report"

        # Try to open the report
        if ("coverage/index.html" | path exists) {
            if $nu.os-info.name == "macos" {
                run-external "open" "coverage/index.html"
            } else if $nu.os-info.name == "linux" {
                run-external "xdg-open" "coverage/index.html"
            }
        }
    } else {
        print_error "Coverage generation failed!"
        print $result.stderr
        exit 1
    }
}

# Show help message
def show_help [] {
    print "
🧪 test.nu - Comprehensive test runner

Usage:
    nu scripts/test.nu [OPTIONS]

Options:
    --all             Run all tests (unit, integration, doc)
    --unit            Run unit tests only
    --integration     Run integration tests only
    --doc             Run documentation tests only
    --wasm            Run WASM-specific tests
    --bench           Run benchmarks
    --coverage        Generate coverage report (requires cargo-tarpaulin)
    --filter PATTERN  Run tests matching pattern
    --nocapture       Show test output (don't capture stdout/stderr)
    --release         Run tests in release mode (optimized)
    --help            Show this help message

Examples:
    # Run all tests
    nu scripts/test.nu

    # Run unit tests only
    nu scripts/test.nu --unit

    # Run tests matching pattern
    nu scripts/test.nu --filter my_test_name

    # Run tests with output
    nu scripts/test.nu --nocapture

    # Generate coverage report
    nu scripts/test.nu --coverage

    # Run WASM tests
    nu scripts/test.nu --wasm

    # Run benchmarks
    nu scripts/test.nu --bench

Prerequisites:
    WASM tests:
        - cargo install wasm-pack
        - Firefox browser installed

    Coverage:
        - cargo install cargo-tarpaulin

    Benchmarks:
        - Benchmark functions in benches/ directory
    "
}

# === Utility Functions ===

def print_header [msg: string] {
    print $"(ansi green_bold)($msg)(ansi reset)"
    print $"(ansi green){'=' * 50}(ansi reset)"
}

def print_step [msg: string] {
    print $"(ansi cyan_bold)▶ ($msg)(ansi reset)"
}

def print_info [msg: string] {
    print $"(ansi blue)ℹ ($msg)(ansi reset)"
}

def print_success [msg: string] {
    print $"(ansi green)✓ ($msg)(ansi reset)"
}

def print_error [msg: string] {
    print $"(ansi red)✗ ($msg)(ansi reset)"
}

def print_warning [msg: string] {
    print $"(ansi yellow)⚠ ($msg)(ansi reset)"
}

def print_command [cmd: list] {
    print $"(ansi purple)$ ($cmd | str join ' ')(ansi reset)"
}
