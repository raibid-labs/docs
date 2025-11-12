#!/usr/bin/env nu

# Discover public repositories in the raibid-labs organization
# Filters based on ignorelist and checks for docs directories
#
# Requirements:
#   - GitHub CLI (gh) installed and authenticated
#   - config/ignorelist.json present
#
# Environment Variables:
#   - GITHUB_ORG: GitHub organization name (default: raibid-labs)
#
# Output:
#   JSON array of repository objects with name, url, docs_path

def main [
    --org: string = "raibid-labs"  # GitHub organization name
    --output: string = "discovered-repos.json"  # Output file path
    --verbose  # Enable verbose logging
] {
    print $"🔍 Discovering repositories in ($org)..."

    # Load ignorelist configuration
    let ignorelist = open config/ignorelist.json

    # Fetch all repositories from the organization using GitHub API
    print "📡 Fetching repositories from GitHub API..."
    let repos = gh api --paginate $"/orgs/($org)/repos" --jq '.[] | {
        name: .name,
        full_name: .full_name,
        html_url: .html_url,
        clone_url: .clone_url,
        description: .description,
        fork: .fork,
        archived: .archived,
        private: .private,
        default_branch: .default_branch,
        updated_at: .updated_at,
        pushed_at: .pushed_at
    }' | lines | each { |line| $line | from json }

    if ($verbose) {
        print $"   Found ($repos | length) total repositories"
    }

    # Filter repositories based on ignorelist
    let filtered = $repos | where {|repo|
        # Exclude if in explicit ignorelist
        if ($repo.name in $ignorelist.repositories) {
            if ($verbose) { print $"   ❌ Excluding ($repo.name): In ignorelist" }
            return false
        }

        # Exclude if matches pattern
        let matches_pattern = $ignorelist.patterns | any {|pattern|
            $repo.name =~ $pattern
        }
        if ($matches_pattern) {
            if ($verbose) { print $"   ❌ Excluding ($repo.name): Matches ignore pattern" }
            return false
        }

        # Exclude forks if configured
        if ($ignorelist.exclude_forks and $repo.fork) {
            if ($verbose) { print $"   ❌ Excluding ($repo.name): Is a fork" }
            return false
        }

        # Exclude archived if configured
        if ($ignorelist.exclude_archived and $repo.archived) {
            if ($verbose) { print $"   ❌ Excluding ($repo.name): Is archived" }
            return false
        }

        # Exclude private if configured
        if ($ignorelist.exclude_private and $repo.private) {
            if ($verbose) { print $"   ❌ Excluding ($repo.name): Is private" }
            return false
        }

        return true
    }

    print $"✅ ($filtered | length) repositories passed filters"

    # Check for docs directory and collect version info
    print "📁 Checking for /docs directories and version info..."
    let with_docs = $filtered | each {|repo|
        let has_docs = (
            gh api $"/repos/($repo.full_name)/contents/docs"
            | complete
            | get exit_code
        ) == 0

        # Get latest release/tag info
        let latest_release = (
            gh api $"/repos/($repo.full_name)/releases/latest"
            | complete
            | if ($in.exit_code == 0) {
                $in.stdout | from json | select tag_name name published_at
            } else {
                null
            }
        )

        # Get latest tag if no release
        let latest_tag = if ($latest_release == null) {
            (
                gh api $"/repos/($repo.full_name)/tags" --jq '.[0].name'
                | complete
                | if ($in.exit_code == 0) { $in.stdout | str trim } else { null }
            )
        } else {
            null
        }

        let version_info = {
            latest_release: $latest_release,
            latest_tag: $latest_tag,
            last_updated: $repo.updated_at,
            last_push: $repo.pushed_at
        }

        if ($has_docs) {
            if ($verbose) { print $"   ✓ ($repo.name): Has /docs directory" }
            $repo
            | insert has_docs true
            | insert docs_path "docs"
            | insert version_info $version_info
        } else {
            if ($verbose) { print $"   ✗ ($repo.name): No /docs directory" }
            $repo
            | insert has_docs false
            | insert docs_path null
            | insert version_info $version_info
        }
    }

    # Filter to only repos with docs if configured
    let final = if ($ignorelist.require_docs_directory) {
        $with_docs | where {|repo| $repo.has_docs }
    } else {
        $with_docs
    }

    print $"📊 Final count: ($final | length) repositories with documentation"

    # Save results
    $final | to json | save --force $output
    print $"💾 Saved results to ($output)"

    # Print summary
    print "\n📋 Summary:"
    $final | select name description has_docs updated_at | table

    return $final
}
