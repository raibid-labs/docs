# Raibid Labs Documentation Hub - Justfile
# Commands to manually trigger documentation sync and build operations

# Set shell for all commands
set shell := ["nu", "-c"]

# Default recipe - show available commands
default:
    @just --list

# Variables
org := "raibid-labs"
content_dir := "content/projects"

# === Repository Discovery ===

# Discover all public repositories with docs in the organization
discover:
    @echo "🔍 Discovering repositories..."
    nu scripts/discover-repos.nu --org {{org}} --verbose

# Discover repositories and show results in JSON format
discover-json:
    @echo "🔍 Discovering repositories (JSON output)..."
    nu scripts/discover-repos.nu --org {{org}} --format json

# === Submodule Management ===

# Sync submodules (add new, update existing, remove deleted)
sync:
    @echo "🔄 Syncing submodules..."
    nu scripts/sync-submodules.nu --verbose

# Update all submodules to latest commits
update:
    @echo "⬆️  Updating submodules to latest..."
    nu scripts/update-docs.nu --generate-index --verbose

# Initialize and update all submodules recursively
init-submodules:
    @echo "📦 Initializing submodules..."
    git submodule update --init --recursive

# Pull latest changes for all submodules
pull-submodules:
    @echo "⬇️  Pulling latest changes for submodules..."
    git submodule update --remote --merge

# Clean and reset all submodules
reset-submodules:
    @echo "🧹 Resetting all submodules..."
    git submodule deinit -f .
    git submodule update --init --recursive

# Remove a specific submodule (usage: just remove-submodule <name>)
remove-submodule name:
    @echo "🗑️  Removing submodule: {{name}}"
    git submodule deinit -f {{content_dir}}/{{name}}
    git rm -f {{content_dir}}/{{name}}
    rm -rf .git/modules/{{content_dir}}/{{name}}

# === Building ===

# Build the Quartz site
build:
    @echo "🔨 Building Quartz site..."
    npx quartz build

# Build and serve the site locally
serve:
    @echo "🚀 Building and serving locally..."
    npx quartz build --serve

# Clean build cache and rebuild
clean-build:
    @echo "🧹 Cleaning build cache..."
    rm -rf public .quartz-cache
    @echo "🔨 Building fresh..."
    npx quartz build

# === Full Pipeline ===

# Run the complete sync and build pipeline
full: discover sync update build
    @echo "✅ Full pipeline completed!"

# Run the complete pipeline with clean build
full-clean: discover sync update clean-build
    @echo "✅ Full clean pipeline completed!"

# === Git Operations ===

# Commit submodule changes
commit-submodules message="chore: update documentation submodules":
    @echo "💾 Committing submodule changes..."
    git add .gitmodules {{content_dir}}
    git diff --cached --quiet || git commit -m "{{message}}"

# Push committed changes
push:
    @echo "⬆️  Pushing changes..."
    git push

# Full git workflow: commit and push
commit-push message="chore: update documentation submodules": (commit-submodules message) push
    @echo "✅ Changes committed and pushed!"

# === Diagnostics ===

# Check the status of all submodules
status:
    @echo "📊 Checking submodule status..."
    git submodule status

# List all configured submodules
list:
    @echo "📋 Configured submodules:"
    git config --file .gitmodules --get-regexp path | awk '{ print $2 }'

# Verify all submodule repositories exist
verify:
    #!/usr/bin/env nu
    echo "🔍 Verifying submodule repositories..."
    let modules = (git config --file .gitmodules --get-regexp url | lines | parse "{key} {url}" | get url)
    for repo in $modules {
        try {
            gh repo view ($repo | str replace "https://github.com/" "" | str replace ".git" "") | ignore
            echo $"✅ ($repo)"
        } catch {
            echo $"❌ ($repo) - NOT FOUND"
        }
    }

# Check for broken submodules
check:
    @echo "🔍 Checking for broken submodules..."
    @git submodule update --init --recursive 2>&1 | grep -i "failed\|error" || echo "✅ All submodules OK"

# === Cleanup ===

# Remove repositories that no longer exist (based on GitHub)
clean-missing:
    #!/usr/bin/env nu
    echo "🧹 Removing submodules for repositories that no longer exist..."
    let broken = ["workspace", "raibid-cli", "raibid-ci", "mop", "locust"]
    for repo in $broken {
        try {
            gh repo view $"raibid-labs/($repo)" | ignore
            echo $"✅ ($repo) still exists"
        } catch {
            echo $"❌ Removing ($repo)..."
            git submodule deinit -f $"{{content_dir}}/($repo)"
            git rm -f $"{{content_dir}}/($repo)"
            rm -rf $".git/modules/{{content_dir}}/($repo)"
        }
    }

# Remove all generated files
clean:
    @echo "🧹 Cleaning generated files..."
    rm -rf public .quartz-cache

# === Dependencies ===

# Install npm dependencies
install:
    @echo "📦 Installing dependencies..."
    npm install

# Update npm dependencies
update-deps:
    @echo "⬆️  Updating dependencies..."
    npm update

# === GitHub Actions ===

# Manually trigger the GitHub Actions workflow
trigger:
    @echo "🚀 Triggering GitHub Actions workflow..."
    gh workflow run sync-and-deploy.yml

# Trigger workflow with clean build
trigger-clean:
    @echo "🚀 Triggering GitHub Actions workflow with clean build..."
    gh workflow run sync-and-deploy.yml -f clean_build=true

# Trigger workflow skipping discovery
trigger-skip-discovery:
    @echo "🚀 Triggering GitHub Actions workflow (skip discovery)..."
    gh workflow run sync-and-deploy.yml -f skip_discovery=true

# View recent workflow runs
runs:
    @echo "📊 Recent workflow runs:"
    gh run list --limit 10

# View the latest workflow run
view-latest:
    @echo "👁️  Viewing latest workflow run..."
    gh run view

# Watch the latest workflow run
watch:
    @echo "👀 Watching latest workflow run..."
    gh run watch

# === Development ===

# Run a local development server with auto-rebuild
dev: serve

# Format all Nushell scripts
format:
    @echo "✨ Formatting Nushell scripts..."
    nu -c "ls scripts/*.nu | each { |f| nu --check $f.name }"

# === Quick Actions ===

# Quick sync: update submodules and build
quick: update build
    @echo "✅ Quick sync completed!"

# Quick fix: reset submodules, sync, and build
fix: reset-submodules sync update build
    @echo "✅ Fixed and rebuilt!"

# === Help ===

# Show detailed help information
help:
    @echo "Raibid Labs Documentation Hub - Manual Commands"
    @echo ""
    @echo "Repository Discovery:"
    @echo "  discover           - Find all public repos with docs"
    @echo "  verify             - Check if all submodule repos exist"
    @echo ""
    @echo "Submodule Management:"
    @echo "  sync               - Add/update/remove submodules"
    @echo "  update             - Update submodules to latest"
    @echo "  pull-submodules    - Pull latest for all submodules"
    @echo "  reset-submodules   - Clean and reinitialize submodules"
    @echo ""
    @echo "Building:"
    @echo "  build              - Build Quartz site"
    @echo "  serve              - Build and serve locally"
    @echo "  clean-build        - Clean cache and rebuild"
    @echo ""
    @echo "Complete Workflows:"
    @echo "  full               - Discover → Sync → Update → Build"
    @echo "  full-clean         - Full workflow with clean build"
    @echo "  quick              - Update → Build (faster)"
    @echo "  fix                - Reset → Sync → Update → Build"
    @echo ""
    @echo "GitHub Actions:"
    @echo "  trigger            - Manually trigger CI/CD"
    @echo "  runs               - View recent workflow runs"
    @echo "  watch              - Watch latest workflow run"
    @echo ""
    @echo "Diagnostics:"
    @echo "  status             - Check submodule status"
    @echo "  check              - Look for broken submodules"
    @echo "  clean-missing      - Remove non-existent repos"
    @echo ""
    @echo "For more info: just --list"
