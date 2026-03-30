#!/usr/bin/env nu
# Run all tests
# Usage: nu scripts/run-tests.nu

def main [] {
    print "Running test suite..."

    let result = (do {
        cargo test --all-features --verbose
    } | complete)

    if $result.exit_code != 0 {
        print $"(ansi red)Tests failed!(ansi reset)"
        print $result.stderr
        exit 1
    }

    print $"(ansi green)All tests passed!(ansi reset)"
}
