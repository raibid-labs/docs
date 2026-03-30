#!/usr/bin/env nu
# Run all CI checks locally
# Usage: nu scripts/ci-local.nu

def main [] {
    print $"(ansi blue_bold)Running local CI checks...(ansi reset)\n"

    # Format check
    print $"(ansi blue)Step 1/3: Format check(ansi reset)"
    nu scripts/check-fmt.nu
    print ""

    # Clippy check
    print $"(ansi blue)Step 2/3: Clippy lints(ansi reset)"
    nu scripts/check-clippy.nu
    print ""

    # Tests
    print $"(ansi blue)Step 3/3: Test suite(ansi reset)"
    nu scripts/run-tests.nu
    print ""

    print $"(ansi green_bold)All CI checks passed!(ansi reset)"
}
