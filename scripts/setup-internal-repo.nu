#!/usr/bin/env nu

# Quick setup script for creating the docs-internal repository
# This script helps bootstrap a companion internal documentation repository
#
# Usage:
#   nu scripts/setup-internal-repo.nu --target /path/to/docs-internal

def main [
    --target: string  # Target directory for docs-internal repo
    --org: string = "raibid-labs"  # GitHub organization
    --dry-run  # Show what would be done without making changes
] {
    print "🚀 Setting up internal documentation repository...\n"

    # Validate target directory
    if ($target | path exists) {
        print $"❌ Error: Target directory ($target) already exists"
        exit 1
    }

    let parent = ($target | path dirname)
    if not ($parent | path exists) {
        print $"❌ Error: Parent directory ($parent) does not exist"
        exit 1
    }

    # Get current repo location
    let source_repo = $env.PWD

    print $"📁 Source repo: ($source_repo)"
    print $"📁 Target location: ($target)\n"

    # Step 1: Create GitHub repository
    print "📝 Step 1: Create private GitHub repository"
    if ($dry_run) {
        print "   [DRY-RUN] Would run:"
        print $"   gh repo create ($org)/docs-internal --private --clone --description 'Internal documentation hub'"
    } else {
        print $"   Creating private repository: ($org)/docs-internal"
        let result = gh repo create $"($org)/docs-internal"
            --private
            --clone
            --description "Internal documentation hub aggregating all raibid-labs repos"
            | complete

        if ($result.exit_code != 0) {
            print $"   ⚠️  Repository may already exist or creation failed"
            print $"   Error: ($result.stderr)"
        } else {
            print "   ✅ Repository created successfully"
        }
    }

    # Step 2: Copy essential files
    print "\n📋 Step 2: Copy configuration and scripts"

    let files_to_copy = [
        "scripts/discover-repos.nu"
        "scripts/sync-submodules.nu"
        "scripts/update-docs.nu"
        "scripts/build-site.nu"
        "config/ignorelist-internal.json"
        "package.json"
        "quartz.config.ts"
        ".gitignore"
        "CLAUDE.md"
    ]

    for file in $files_to_copy {
        let source = $"($source_repo)/($file)"
        let dest_path = $"($target)/($file)"

        if ($dry_run) {
            print $"   [DRY-RUN] Would copy: ($file)"
        } else {
            if ($source | path exists) {
                mkdir ($dest_path | path dirname)
                cp $source $dest_path
                print $"   ✅ Copied: ($file)"
            } else {
                print $"   ⚠️  Skipped (not found): ($file)"
            }
        }
    }

    # Step 3: Update configuration for internal use
    print "\n⚙️  Step 3: Update configuration for internal repository"

    if ($dry_run) {
        print "   [DRY-RUN] Would update ignorelist configuration"
        print "   [DRY-RUN] Would update Quartz configuration"
    } else {
        # Move ignorelist-internal.json to ignorelist.json
        if ($"($target)/config/ignorelist-internal.json" | path exists) {
            mv $"($target)/config/ignorelist-internal.json" $"($target)/config/ignorelist.json"
            print "   ✅ Updated ignorelist configuration"
        }

        # Update quartz.config.ts title
        let quartz_config = $"($target)/quartz.config.ts"
        if ($quartz_config | path exists) {
            let content = open $quartz_config | str replace "Raibid Labs Documentation" "Raibid Labs Internal Documentation"
            $content | save --force $quartz_config
            print "   ✅ Updated Quartz configuration title"
        }
    }

    # Step 4: Copy Quartz installation
    print "\n📦 Step 4: Copy Quartz installation"

    if ($dry_run) {
        print "   [DRY-RUN] Would copy Quartz directory"
    } else {
        let quartz_source = $"($source_repo)/quartz"
        let quartz_dest = $"($target)/quartz"

        if ($quartz_source | path exists) {
            cp -r $quartz_source $quartz_dest
            print "   ✅ Copied Quartz installation"
        } else {
            print "   ⚠️  Quartz not found, will need to be installed separately"
        }
    }

    # Step 5: Create directory structure
    print "\n📁 Step 5: Create directory structure"

    let dirs = [
        "docs/content/projects"
        "docs/content/guides"
        "docs/content/architecture"
        "docs/content/infrastructure"
        ".github/workflows"
    ]

    for dir in $dirs {
        if ($dry_run) {
            print $"   [DRY-RUN] Would create: ($dir)"
        } else {
            mkdir $"($target)/($dir)"
            print $"   ✅ Created: ($dir)"
        }
    }

    # Step 6: Create initial documentation
    print "\n📝 Step 6: Create initial documentation files"

    if ($dry_run) {
        print "   [DRY-RUN] Would create index.md"
        print "   [DRY-RUN] Would create GitHub Actions workflow"
    } else {
        # Create docs/index.md
        let index_content = [
            "---"
            "title: Raibid Labs Internal Documentation"
            "description: Internal documentation hub for raibid-labs (public + private repos)"
            "tags: [home, documentation, internal]"
            "---"
            ""
            "# Welcome to Raibid Labs Internal Documentation"
            ""
            "This is the **internal** documentation hub for the raibid-labs organization."
            ""
            "⚠️ **This site contains confidential information. Access is restricted to organization members.**"
            ""
            "## Quick Links"
            ""
            "- [[content/projects/index|All Projects]]"
            "- [[content/guides/getting-started|Getting Started]]"
            "- [Public Documentation](https://raibid-labs.github.io/docs) (external link)"
            ""
            "## What's Different?"
            ""
            "This internal hub includes:"
            "- All public repository documentation"
            "- Private repository documentation"
            "- Internal guides and processes"
            "- Confidential architecture documentation"
        ]

        $index_content | str join "\n" | save --force $"($target)/docs/index.md"
        print "   ✅ Created docs/index.md"

        # Copy workflow from source
        let workflow_source = $"($source_repo)/.github/workflows/sync-and-deploy.yml"
        let workflow_dest = $"($target)/.github/workflows/sync-and-deploy.yml"

        if ($workflow_source | path exists) {
            cp $workflow_source $workflow_dest
            print "   ✅ Created GitHub Actions workflow"
        }
    }

    # Step 7: Initialize git and install dependencies
    print "\n🔧 Step 7: Initialize repository"

    if ($dry_run) {
        print "   [DRY-RUN] Would initialize git repository"
        print "   [DRY-RUN] Would install npm dependencies"
    } else {
        cd $target

        # Git initialization
        git init | complete
        git add -A
        git commit -m "Initial commit: Internal documentation repository setup"

        # Add remote
        git remote add origin $"https://github.com/($org)/docs-internal.git"

        print "   ✅ Initialized git repository"

        # Install dependencies
        print "   📦 Installing npm dependencies..."
        npm install | complete
        print "   ✅ Dependencies installed"
    }

    # Print summary and next steps
    print "\n" + ("=" * 70)
    print "✅ Internal documentation repository setup complete!\n"

    print "📊 Summary:"
    print $"   Repository: ($org)/docs-internal (private)"
    print $"   Location: ($target)"
    print $"   Configuration: Internal repos enabled\n"

    print "🚀 Next Steps:"
    print $"   1. cd ($target)"
    print "   2. Review and update config/ignorelist.json if needed"
    print "   3. Run repository discovery:"
    print "      npm run discover"
    print "   4. Sync submodules:"
    print "      npm run sync"
    print "   5. Build the site:"
    print "      npm run build"
    print "   6. Test locally:"
    print "      npm run dev"
    print "   7. Push to GitHub:"
    print "      git push -u origin main"
    print ""
    print "📚 Documentation:"
    print "   See docs/content/guides/private-docs-setup.md for detailed setup"
    print ""
    print "🔒 Deployment Options:"
    print "   - Local builds (most secure)"
    print "   - GitHub Actions artifacts"
    print "   - Cloudflare Pages + Access (recommended for teams)"
    print ""

    if ($dry_run) {
        print "💡 This was a dry-run. Re-run without --dry-run to apply changes."
    }
}
