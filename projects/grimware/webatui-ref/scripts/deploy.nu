#!/usr/bin/env nu

# deploy.nu - Deploy WASM builds to GitHub Pages or other hosting
#
# Usage:
#   nu scripts/deploy.nu [OPTIONS]
#
# Options:
#   --target TARGET   Deployment target (github-pages, netlify, vercel)
#   --branch BRANCH   Git branch for GitHub Pages (default: gh-pages)
#   --dir DIR         Directory to deploy (default: pkg)
#   --message MSG     Commit message (default: "Deploy WASM build")
#   --dry-run         Show what would be deployed without actually deploying
#   --help            Show this help message

def main [
    --target: string = "github-pages" # Deployment target
    --branch: string = "gh-pages"     # Git branch for GitHub Pages
    --dir: string = "pkg"             # Directory to deploy
    --message: string = "Deploy WASM build" # Commit message
    --dry-run                         # Dry run mode
    --help                            # Show help message
] {
    if $help {
        show_help
        return
    }

    print_header "🚀 Deployment Automation"

    # Check if deployment directory exists
    if not ($dir | path exists) {
        print_error $"Deployment directory not found: ($dir)"
        print_info "Build your project first with: just build-wasm"
        exit 1
    }

    match $target {
        "github-pages" => { deploy_github_pages $branch $dir $message $dry_run }
        "netlify" => { deploy_netlify $dir $dry_run }
        "vercel" => { deploy_vercel $dir $dry_run }
        _ => {
            print_error $"Unknown deployment target: ($target)"
            print_info "Supported targets: github-pages, netlify, vercel"
            exit 1
        }
    }
}

# Deploy to GitHub Pages
def deploy_github_pages [branch: string, dir: string, message: string, dry_run: bool] {
    print_step "Deploying to GitHub Pages"

    # Check if we're in a git repository
    if not (".git" | path exists) {
        print_error "Not a git repository!"
        print_info "Initialize git with: git init"
        exit 1
    }

    # Check if gh-pages branch exists
    let branch_exists = (git branch --list $branch | str trim | str length) > 0

    if $dry_run {
        print_warning "DRY RUN MODE - No actual deployment will occur"
        print_info $"Would deploy ($dir) to branch ($branch)"
        print_info $"Commit message: ($message)"

        # Show files that would be deployed
        print_step "Files to deploy:"
        ls $dir | select name size | print
        return
    }

    # Create a temporary directory for deployment
    let temp_dir = (mktemp -d)
    print_info $"Using temporary directory: ($temp_dir)"

    try {
        # Copy deployment files to temp directory
        print_step "Copying files to temporary directory"
        cp -r ($dir | path join "*") $temp_dir

        # Initialize git in temp directory
        cd $temp_dir
        git init
        git checkout -b $branch

        # Add all files
        git add -A

        # Create commit
        git commit -m $message

        # Get remote URL from original repo
        cd -
        let remote_url = (git remote get-url origin | str trim)

        # Push to gh-pages branch
        cd $temp_dir
        git remote add origin $remote_url

        print_step $"Pushing to ($branch) branch"
        git push -f origin $branch

        print_success "Deployed to GitHub Pages!"
        print_info $"Your site should be available at: https://<username>.github.io/<repo>"

    } catch {
        print_error "Deployment failed!"
        print_info $"Error: ($error)"
    }

    # Cleanup
    print_step "Cleaning up temporary directory"
    rm -rf $temp_dir
}

# Deploy to Netlify
def deploy_netlify [dir: string, dry_run: bool] {
    print_step "Deploying to Netlify"

    # Check if Netlify CLI is installed
    let netlify_installed = (which netlify | length) > 0

    if not $netlify_installed {
        print_error "Netlify CLI is not installed!"
        print_info "Install with: npm install -g netlify-cli"
        exit 1
    }

    if $dry_run {
        print_warning "DRY RUN MODE - No actual deployment will occur"
        print_info $"Would deploy ($dir) to Netlify"

        # Show files that would be deployed
        print_step "Files to deploy:"
        ls $dir | select name size | print
        return
    }

    # Deploy to Netlify
    print_step "Deploying to Netlify..."
    let cmd = ["netlify", "deploy", "--dir", $dir, "--prod"]

    print_command $cmd
    run-external $cmd.0 ...$cmd.1..

    print_success "Deployed to Netlify!"
}

# Deploy to Vercel
def deploy_vercel [dir: string, dry_run: bool] {
    print_step "Deploying to Vercel"

    # Check if Vercel CLI is installed
    let vercel_installed = (which vercel | length) > 0

    if not $vercel_installed {
        print_error "Vercel CLI is not installed!"
        print_info "Install with: npm install -g vercel"
        exit 1
    }

    if $dry_run {
        print_warning "DRY RUN MODE - No actual deployment will occur"
        print_info $"Would deploy ($dir) to Vercel"

        # Show files that would be deployed
        print_step "Files to deploy:"
        ls $dir | select name size | print
        return
    }

    # Deploy to Vercel
    print_step "Deploying to Vercel..."
    cd $dir

    let cmd = ["vercel", "--prod"]

    print_command $cmd
    run-external $cmd.0 ...$cmd.1..

    print_success "Deployed to Vercel!"
}

# Show help message
def show_help [] {
    print "
🚀 deploy.nu - Deployment automation for WASM builds

Usage:
    nu scripts/deploy.nu [OPTIONS]

Options:
    --target TARGET   Deployment target (github-pages, netlify, vercel)
                      Default: github-pages
    --branch BRANCH   Git branch for GitHub Pages (default: gh-pages)
    --dir DIR         Directory to deploy (default: pkg)
    --message MSG     Commit message for GitHub Pages
                      Default: 'Deploy WASM build'
    --dry-run         Show what would be deployed without deploying
    --help            Show this help message

Deployment Targets:
    github-pages      Deploy to GitHub Pages (free, automatic HTTPS)
    netlify          Deploy to Netlify (requires Netlify CLI)
    vercel           Deploy to Vercel (requires Vercel CLI)

Examples:
    # Deploy to GitHub Pages
    nu scripts/deploy.nu

    # Deploy to Netlify
    nu scripts/deploy.nu --target netlify

    # Deploy to Vercel
    nu scripts/deploy.nu --target vercel

    # Dry run to see what would be deployed
    nu scripts/deploy.nu --dry-run

    # Deploy with custom branch and message
    nu scripts/deploy.nu --branch main --message 'Update site'

Prerequisites:
    GitHub Pages:
        - Git repository with remote origin configured
        - GitHub repository settings > Pages enabled

    Netlify:
        - npm install -g netlify-cli
        - netlify login (first time only)

    Vercel:
        - npm install -g vercel
        - vercel login (first time only)
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
