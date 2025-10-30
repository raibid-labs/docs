#!/usr/bin/env nu

# Build orchestration script for the documentation site
# Coordinates all steps: discovery, sync, update, and build
#
# Requirements:
#   - All other scripts (discover-repos.nu, sync-submodules.nu, update-docs.nu)
#   - Quartz installed (npx quartz)
#   - Node.js v22+
#
# Output:
#   Built static site in public/ directory

def main [
    --skip-discovery  # Skip repository discovery step
    --skip-sync  # Skip submodule synchronization
    --skip-update  # Skip documentation update
    --serve  # Start local dev server after build
    --clean  # Clean build (remove cache)
    --verbose  # Enable verbose logging
] {
    print "🚀 Starting documentation site build pipeline...\n"

    let start_time = date now

    # Step 1: Check prerequisites
    print "✓ Checking prerequisites..."
    check_prerequisites

    # Step 2: Discover repositories
    if not ($skip_discovery) {
        print "\n📡 Step 1/4: Discovering repositories..."
        nu scripts/discover-repos.nu (if ($verbose) { ["--verbose"] } else { [] })
    } else {
        print "\n⏭️  Skipping repository discovery"
    }

    # Step 3: Sync submodules
    if not ($skip_sync) {
        print "\n🔄 Step 2/4: Synchronizing submodules..."
        nu scripts/sync-submodules.nu (if ($verbose) { ["--verbose"] } else { [] })
    } else {
        print "\n⏭️  Skipping submodule synchronization"
    }

    # Step 4: Update documentation
    if not ($skip_update) {
        print "\n📚 Step 3/4: Updating documentation..."
        nu scripts/update-docs.nu --generate-index (if ($verbose) { ["--verbose"] } else { [] })
    } else {
        print "\n⏭️  Skipping documentation update"
    }

    # Step 5: Build with Quartz
    print "\n🏗️  Step 4/4: Building site with Quartz..."

    if ($clean) {
        print "   🧹 Cleaning build cache..."
        if ("public" | path exists) { rm -rf public }
        if (".quartz-cache" | path exists) { rm -rf .quartz-cache }
    }

    let build_result = if ($serve) {
        print "   🌐 Starting development server..."
        npx quartz build --serve
    } else {
        npx quartz build
    } | complete

    if ($build_result.exit_code != 0) {
        print $"\n❌ Build failed with error:"
        print $"($build_result.stderr)"
        exit 1
    }

    # Calculate build time
    let end_time = date now
    let duration = $end_time - $start_time

    # Print summary
    print "\n" + ("=" | fill -c "=" -w 60)
    print "✅ Build pipeline completed successfully!"
    print ("=" | fill -c "=" -w 60)
    print $"⏱️  Total time: ($duration)"

    if not ($serve) {
        print $"\n📦 Output: public/ directory"
        print "💡 To preview: npx quartz build --serve"
        print "🚀 To deploy: Follow your deployment pipeline"
    }
}

# Check if all prerequisites are installed
def check_prerequisites [] {
    let mut missing = []

    # Check Node.js
    let node_version = node --version | complete
    if ($node_version.exit_code != 0) {
        $missing = ($missing | append "Node.js (v22+ required)")
    } else {
        let version = $node_version.stdout | str trim | str substring 1..
        let major = $version | split row "." | first | into int
        if ($major < 22) {
            print $"⚠️  Warning: Node.js v22+ required, found v($version)"
        }
    }

    # Check Git
    if (git --version | complete | get exit_code) != 0 {
        $missing = ($missing | append "Git")
    }

    # Check GitHub CLI
    if (gh --version | complete | get exit_code) != 0 {
        $missing = ($missing | append "GitHub CLI (gh)")
    }

    # Check for config files
    if not ("config/ignorelist.json" | path exists) {
        $missing = ($missing | append "config/ignorelist.json")
    }

    if ($missing | length) > 0 {
        print "❌ Missing prerequisites:"
        $missing | each {|item| print $"   - ($item)" }
        exit 1
    }

    print "   All prerequisites satisfied"
}
