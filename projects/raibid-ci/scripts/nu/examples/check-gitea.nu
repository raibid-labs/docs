#!/usr/bin/env nu
# Raibid-CI Gitea Health Check Example
# Demonstrates using gitea module to check Gitea health

# Source environment and config
source ../env.nu
source ../config.nu

# Import gitea module
use ../modules/gitea.nu *

def main [] {
    print "=== Raibid-CI Gitea Health Check ==="
    print ""

    let gitea_url = if "RAIBID_GITEA_URL" in $env {
        $env.RAIBID_GITEA_URL
    } else {
        "http://localhost:3000"
    }

    print $"Gitea URL: ($gitea_url)"
    print ""

    # Check connection
    if not (gitea-check-connection $gitea_url) {
        log-error "Cannot connect to Gitea. Is Gitea running?"
        exit 1
    }

    print ""

    # Get version
    log-info "Version Information:"
    let version_info = (gitea-version $gitea_url)
    print $"  Version: ($version_info.version)"

    print ""

    # Check if authenticated
    if "GITEA_TOKEN" in $env {
        log-info "Authentication: Token found"

        try {
            let user = (gitea-user)
            print $"  User: ($user.login)"
            print $"  Email: ($user.email)"

            print ""

            # List repositories
            log-info "Repositories:"
            let repos = (gitea-list-repos --limit 10)

            if ($repos | length) > 0 {
                $repos | select name private url | table
            } else {
                print "  (no repositories)"
            }

            print ""

            # List organizations
            log-info "Organizations:"
            let orgs = (gitea-list-orgs)

            if ($orgs | length) > 0 {
                $orgs | table
            } else {
                print "  (no organizations)"
            }
        } catch {
            log-warning "Token may be invalid or expired"
        }
    } else {
        log-warning "No GITEA_TOKEN environment variable set"
        print "  Set GITEA_TOKEN to enable authenticated requests"
    }

    print ""
    log-success "Gitea health check complete!"
}

main
