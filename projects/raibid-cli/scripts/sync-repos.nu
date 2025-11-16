#!/usr/bin/env nu

# Synchronize all raibid-labs repositories
# This is a nushell wrapper around the raibid CLI

def main [
    --workspace: string = "~/raibid-labs"  # Workspace root directory
    --concurrency: int = 5  # Number of concurrent operations
    --dry-run  # Perform dry run without actual changes
    --filter: string  # Filter repositories by pattern
] {
    print "=== Synchronizing raibid-labs repositories ==="

    # Build the raibid command
    mut command = ["cargo", "run", "--bin", "raibid", "--", "sync", "--all"]

    if $dry_run {
        $command = ($command | append "--dry-run")
    }

    if $filter != null {
        $command = ($command | append ["--filter", $filter])
    }

    $command = ($command | append ["--concurrency", ($concurrency | into string)])

    # Execute the command
    print $"Running: ($command | str join ' ')"
    ^$command.0 ...$command.1..
}
