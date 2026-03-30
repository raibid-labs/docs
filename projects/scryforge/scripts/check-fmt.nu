#!/usr/bin/env nu
# Check Rust formatting
# Usage: nu scripts/check-fmt.nu

def main [] {
    print "Checking Rust formatting..."

    let result = (do { cargo fmt --all -- --check } | complete)

    if $result.exit_code != 0 {
        print $"(ansi red)Formatting check failed!(ansi reset)"
        print $result.stderr
        exit 1
    }

    print $"(ansi green)All files are properly formatted!(ansi reset)"
}
