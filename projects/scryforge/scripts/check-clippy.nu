#!/usr/bin/env nu
# Run Clippy lints
# Usage: nu scripts/check-clippy.nu

def main [] {
    print "Running Clippy lints..."

    let result = (do {
        cargo clippy --all-targets --all-features -- -D warnings
    } | complete)

    if $result.exit_code != 0 {
        print $"(ansi red)Clippy found issues!(ansi reset)"
        print $result.stderr
        exit 1
    }

    print $"(ansi green)No Clippy warnings found!(ansi reset)"
}
