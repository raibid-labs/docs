#!/usr/bin/env nu

# Discover and catalog repositories in the raibid-labs GitHub organization
# This script uses the GitHub CLI (gh) to fetch repository information

def main [
    --org: string = "raibid-labs"  # GitHub organization name
    --output: string = "repos.json"  # Output JSON file
    --verbose  # Enable verbose logging
] {
    if $verbose {
        print $"Discovering repositories in ($org)..."
    }

    # Check if gh CLI is available
    if (which gh | is-empty) {
        error make {msg: "GitHub CLI (gh) is not installed. Please install it first."}
    }

    # Fetch all repositories from the organization
    let repos = (
        gh api $"/orgs/($org)/repos" --paginate
        | from json
        | each { |repo|
            {
                name: $repo.name,
                full_name: $repo.full_name,
                description: $repo.description,
                clone_url: $repo.clone_url,
                ssh_url: $repo.ssh_url,
                default_branch: $repo.default_branch,
                private: $repo.private,
                fork: $repo.fork,
                archived: $repo.archived,
                language: $repo.language,
                stargazers_count: $repo.stargazers_count,
                forks_count: $repo.forks_count,
                updated_at: $repo.updated_at,
                pushed_at: $repo.pushed_at,
                topics: $repo.topics,
                has_docs: (check_has_docs $repo.name $org)
            }
        }
    )

    if $verbose {
        print $"Found ($repos | length) repositories"
    }

    # Save to JSON file
    $repos | to json | save --force $output

    if $verbose {
        print $"Saved repository data to ($output)"
    }

    # Display summary table
    $repos
    | select name language stargazers_count archived has_docs updated_at
    | sort-by updated_at --reverse
}

# Check if a repository has a /docs directory
def check_has_docs [
    repo_name: string
    org: string
] {
    let result = (
        gh api $"/repos/($org)/($repo_name)/contents/docs"
        | complete
    )

    $result.exit_code == 0
}
