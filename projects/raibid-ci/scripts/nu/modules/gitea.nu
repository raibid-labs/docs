# Raibid-CI Gitea Utility Module
# Helper functions for interacting with Gitea API
#
# Usage:
#   use gitea.nu *
#   gitea-check

# Default Gitea URL from environment or localhost
export def gitea-url [] {
    if "RAIBID_GITEA_URL" in $env {
        $env.RAIBID_GITEA_URL
    } else {
        "http://localhost:3000"
    }
}

# Get Gitea API URL
export def gitea-api-url [] {
    $"(gitea-url)/api/v1"
}

# Check if curl is available
export def gitea-check [] {
    if (which curl | is-empty) {
        error make {msg: "curl not found. Please install curl."}
    }
}

# Check Gitea connectivity
export def gitea-check-connection [
    url?: string  # Gitea URL (optional, uses default if not provided)
] {
    gitea-check

    let base_url = if $url == null { (gitea-url) } else { $url }
    print $"Checking Gitea connection at ($base_url)..."

    try {
        let result = (curl -f -s $"($base_url)/api/v1/version" | complete)
        if $result.exit_code == 0 {
            let version = ($result.stdout | from json | get version)
            print $"(ansi green)✓(ansi reset) Connected to Gitea version ($version)"
            return true
        } else {
            print $"(ansi red)✗(ansi reset) Cannot connect to Gitea"
            return false
        }
    } catch {
        print $"(ansi red)✗(ansi reset) Error connecting to Gitea"
        return false
    }
}

# Get Gitea version
export def gitea-version [
    url?: string  # Gitea URL
] {
    gitea-check

    let api_url = if $url == null { (gitea-api-url) } else { $"($url)/api/v1" }

    curl -s $"($api_url)/version" | from json
}

# Make authenticated API request
export def gitea-api [
    endpoint: string  # API endpoint (e.g., "/repos")
    token?: string    # API token (optional, uses GITEA_TOKEN env var)
    --method (-m): string = "GET"  # HTTP method
    --data (-d): string  # Request body (JSON)
] {
    gitea-check

    let api_url = (gitea-api-url)
    let url = $"($api_url)($endpoint)"

    mut args = ["-s", "-X", $method]

    # Add authentication header if token provided or in environment
    let auth_token = if $token != null {
        $token
    } else if "GITEA_TOKEN" in $env {
        $env.GITEA_TOKEN
    } else {
        null
    }

    if $auth_token != null {
        $args = ($args | append ["-H", $"Authorization: token ($auth_token)"])
    }

    # Add content type for POST/PUT requests
    if $method in ["POST", "PUT", "PATCH"] {
        $args = ($args | append ["-H", "Content-Type: application/json"])
    }

    # Add data if provided
    if $data != null {
        $args = ($args | append ["-d", $data])
    }

    $args = ($args | append $url)

    let result = (curl ...$args | from json)
    $result
}

# List repositories
export def gitea-list-repos [
    token?: string  # API token
    --limit (-l): int = 50  # Maximum number of repos
] {
    gitea-api "/user/repos" $token --method GET |
        each {|repo|
            {
                name: $repo.name
                full_name: $repo.full_name
                private: $repo.private
                url: $repo.html_url
                clone_url: $repo.clone_url
            }
        } |
        first $limit
}

# Get repository information
export def gitea-get-repo [
    owner: string  # Repository owner
    repo: string   # Repository name
    token?: string # API token
] {
    gitea-api $"/repos/($owner)/($repo)" $token
}

# Create repository
export def gitea-create-repo [
    name: string         # Repository name
    token?: string       # API token
    --description (-d): string  # Repository description
    --private (-p)       # Make repository private
] {
    gitea-check

    mut data = {
        name: $name
        auto_init: false
    }

    if $description != null {
        $data = ($data | insert description $description)
    }

    if $private {
        $data = ($data | insert private true)
    }

    gitea-api "/user/repos" $token --method POST --data ($data | to json)
}

# Delete repository (DANGER!)
export def gitea-delete-repo [
    owner: string   # Repository owner
    repo: string    # Repository name
    token?: string  # API token
    --confirm       # Confirmation flag
] {
    if not $confirm {
        print $"(ansi red)⚠ WARNING:(ansi reset) This will permanently delete ($owner)/($repo)!"
        print "Use --confirm flag if you're sure."
        return
    }

    gitea-api $"/repos/($owner)/($repo)" $token --method DELETE
    print $"(ansi green)✓(ansi reset) Repository deleted: ($owner)/($repo)"
}

# List branches
export def gitea-list-branches [
    owner: string   # Repository owner
    repo: string    # Repository name
    token?: string  # API token
] {
    gitea-api $"/repos/($owner)/($repo)/branches" $token |
        each {|branch|
            {
                name: $branch.name
                commit: $branch.commit.id
                protected: $branch.protected
            }
        }
}

# List tags
export def gitea-list-tags [
    owner: string   # Repository owner
    repo: string    # Repository name
    token?: string  # API token
] {
    gitea-api $"/repos/($owner)/($repo)/tags" $token |
        each {|tag|
            {
                name: $tag.name
                commit: $tag.commit.sha
            }
        }
}

# Create webhook
export def gitea-create-webhook [
    owner: string      # Repository owner
    repo: string       # Repository name
    url: string        # Webhook URL
    token?: string     # API token
    --events (-e): list<string> = ["push"]  # Events to trigger webhook
] {
    gitea-check

    let data = {
        type: "gitea"
        config: {
            url: $url
            content_type: "json"
        }
        events: $events
        active: true
    }

    gitea-api $"/repos/($owner)/($repo)/hooks" $token --method POST --data ($data | to json)
}

# List webhooks
export def gitea-list-webhooks [
    owner: string   # Repository owner
    repo: string    # Repository name
    token?: string  # API token
] {
    gitea-api $"/repos/($owner)/($repo)/hooks" $token |
        each {|hook|
            {
                id: $hook.id
                type: $hook.type
                url: $hook.config.url
                events: $hook.events
                active: $hook.active
            }
        }
}

# Create organization
export def gitea-create-org [
    name: string         # Organization name
    token?: string       # API token
    --description (-d): string  # Organization description
] {
    gitea-check

    mut data = {
        username: $name
    }

    if $description != null {
        $data = ($data | insert description $description)
    }

    gitea-api "/orgs" $token --method POST --data ($data | to json)
}

# List organizations
export def gitea-list-orgs [
    token?: string  # API token
] {
    gitea-api "/user/orgs" $token |
        each {|org|
            {
                name: $org.username
                full_name: $org.full_name
                description: $org.description
            }
        }
}

# Create release
export def gitea-create-release [
    owner: string      # Repository owner
    repo: string       # Repository name
    tag: string        # Tag name
    token?: string     # API token
    --name (-n): string  # Release name
    --body (-b): string  # Release notes
    --draft (-d)       # Create as draft
    --prerelease (-p)  # Mark as prerelease
] {
    gitea-check

    mut data = {
        tag_name: $tag
    }

    if $name != null {
        $data = ($data | insert name $name)
    }

    if $body != null {
        $data = ($data | insert body $body)
    }

    if $draft {
        $data = ($data | insert draft true)
    }

    if $prerelease {
        $data = ($data | insert prerelease true)
    }

    gitea-api $"/repos/($owner)/($repo)/releases" $token --method POST --data ($data | to json)
}

# List releases
export def gitea-list-releases [
    owner: string   # Repository owner
    repo: string    # Repository name
    token?: string  # API token
] {
    gitea-api $"/repos/($owner)/($repo)/releases" $token |
        each {|release|
            {
                id: $release.id
                tag: $release.tag_name
                name: $release.name
                draft: $release.draft
                prerelease: $release.prerelease
                published: $release.published_at
            }
        }
}

# Search repositories
export def gitea-search-repos [
    query: string   # Search query
    token?: string  # API token
    --limit (-l): int = 20  # Maximum results
] {
    gitea-api $"/repos/search?q=($query)&limit=($limit)" $token |
        get data |
        each {|repo|
            {
                name: $repo.name
                full_name: $repo.full_name
                description: $repo.description
                stars: $repo.stars_count
                forks: $repo.forks_count
            }
        }
}

# Get user information
export def gitea-user [
    token?: string  # API token
] {
    gitea-api "/user" $token
}

# Mirror repository from external source
export def gitea-mirror-repo [
    clone_url: string    # Source repository URL
    repo_name: string    # New repository name
    token?: string       # API token
    --description (-d): string  # Repository description
    --private (-p)       # Make repository private
] {
    gitea-check

    mut data = {
        clone_addr: $clone_url
        repo_name: $repo_name
        mirror: true
    }

    if $description != null {
        $data = ($data | insert description $description)
    }

    if $private {
        $data = ($data | insert private true)
    }

    gitea-api "/repos/migrate" $token --method POST --data ($data | to json)
}
