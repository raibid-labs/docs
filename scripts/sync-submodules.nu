#!/usr/bin/env nu

# Synchronize git submodules based on discovered repositories
# Adds new submodules, updates existing ones, and removes stale ones
#
# Requirements:
#   - discovered-repos.json (from discover-repos.nu)
#   - Git with submodule support
#
# Output:
#   Updates .gitmodules and submodule directories

def main [
    --input: string = "discovered-repos.json"  # Input file from discover-repos.nu
    --target: string = "content/projects"  # Target directory for submodules
    --dry-run  # Show what would be done without making changes
    --verbose  # Enable verbose logging
] {
    print "🔄 Synchronizing submodules..."

    # Load discovered repositories
    if not ($input | path exists) {
        print $"❌ Error: Input file ($input) not found"
        print "   Run discover-repos.nu first to generate the repository list"
        exit 1
    }

    let discovered = open $input | where {|repo| $repo.has_docs }
    print $"📊 Found ($discovered | length) repositories with docs"

    # Ensure target directory exists
    if not ($dry_run) {
        mkdir $target
    }

    # Get list of existing submodules
    let existing_submodules = if (".gitmodules" | path exists) {
        git config --file .gitmodules --get-regexp path
        | lines
        | parse "submodule.{name}.path {path}"
        | where {|sm| $sm.path | str starts-with $target }
    } else {
        []
    }

    if ($verbose) {
        print $"   Currently tracking ($existing_submodules | length) submodules"
    }

    # Process each discovered repository
    mut added = 0
    mut updated = 0
    mut skipped = 0

    for repo in $discovered {
        let submodule_path = $"($target)/($repo.name)"
        let exists = $existing_submodules | any {|sm| $sm.path == $submodule_path }

        if ($exists) {
            # Update existing submodule
            print $"🔄 Updating submodule: ($repo.name)"
            if not ($dry_run) {
                git submodule update --remote --merge $submodule_path
                | complete
                | if ($in.exit_code == 0) {
                    $updated = $updated + 1
                    if ($verbose) { print $"   ✅ Updated ($repo.name)" }
                } else {
                    print $"   ⚠️  Failed to update ($repo.name): ($in.stderr)"
                }
            } else {
                print $"   [DRY-RUN] Would update ($submodule_path)"
                $updated = $updated + 1
            }
        } else {
            # Add new submodule
            print $"➕ Adding new submodule: ($repo.name)"
            if not ($dry_run) {
                git submodule add --force $repo.clone_url $submodule_path
                | complete
                | if ($in.exit_code == 0) {
                    # Configure submodule to track specific path
                    git config -f .gitmodules $"submodule.($submodule_path).sparse-checkout" "docs/*"
                    $added = $added + 1
                    if ($verbose) { print $"   ✅ Added ($repo.name)" }
                } else {
                    print $"   ⚠️  Failed to add ($repo.name): ($in.stderr)"
                }
            } else {
                print $"   [DRY-RUN] Would add ($repo.clone_url) to ($submodule_path)"
                $added = $added + 1
            }
        }
    }

    # Stage .gitmodules changes before removing stale submodules
    if not ($dry_run) and ($added > 0) {
        git add .gitmodules
    }

    # Identify and remove stale submodules
    let discovered_paths = $discovered | each {|repo| $"($target)/($repo.name)" }
    let stale = $existing_submodules | where {|sm|
        not ($sm.path in $discovered_paths)
    }

    mut removed = 0
    if ($stale | length) > 0 {
        print $"\n🗑️  Found ($stale | length) stale submodules"
        for sm in $stale {
            print $"   Removing: ($sm.path)"
            if not ($dry_run) {
                git submodule deinit -f $sm.path
                git rm -f $sm.path
                rm -rf $".git/modules/($sm.path)"
                $removed = $removed + 1
                if ($verbose) { print $"      ✅ Removed ($sm.path)" }
            } else {
                print $"      [DRY-RUN] Would remove ($sm.path)"
                $removed = $removed + 1
            }
        }
    }

    # Initialize and update all submodules
    if not ($dry_run) and ($added > 0) {
        print "\n🔧 Initializing submodules..."
        git submodule update --init --recursive
    }

    # Print summary
    print "\n📊 Synchronization Summary:"
    print $"   ➕ Added: ($added)"
    print $"   🔄 Updated: ($updated)"
    print $"   🗑️  Removed: ($removed)"
    print $"   ⏭️  Skipped: ($skipped)"

    if ($dry_run) {
        print "\n💡 This was a dry-run. Re-run without --dry-run to apply changes."
    } else {
        print "\n✅ Submodule synchronization complete!"
        print "💡 Don't forget to commit the changes to .gitmodules"
    }
}
