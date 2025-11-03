#!/usr/bin/env nu

# Update documentation from submodules and organize for Quartz
# Creates index files and navigation structure
#
# Requirements:
#   - Git submodules already configured
#   - Quartz project structure
#
# Output:
#   Updated content directory with organized docs

def main [
    --source: string = "content/projects"  # Submodules directory
    --verbose  # Enable verbose logging
    --generate-index  # Generate index.md files for each project
] {
    print "📚 Updating documentation from submodules..."

    # Update all submodules to latest
    print "🔄 Pulling latest changes from all submodules..."
    git submodule update --remote --merge | complete

    if ($verbose) {
        git submodule status | lines | each {|line| print $"   ($line)" }
    }

    # Get list of project submodules
    let projects = ls $source
    | where {|item| $item.type == "dir" }
    | get name
    | path basename

    print $"📁 Processing ($projects | length) project directories..."

    mut processed = 0
    mut failed = 0

    for project in $projects {
        let project_path = $"($source)/($project)"
        let docs_path = $"($project_path)/docs"

        if ($docs_path | path exists) {
            print $"✓ Processing: ($project)"

            # Generate index.md if requested and doesn't exist
            if ($generate_index) {
                let index_path = $"($docs_path)/index.md"
                if not ($index_path | path exists) {
                    if ($verbose) { print $"   📝 Generating index for ($project)" }
                    generate_project_index $project $docs_path
                }
            }

            # Validate markdown files
            let md_files = ls $docs_path | where name =~ '\.md$' | length
            if ($verbose) {
                print $"   📄 Found ($md_files) markdown files"
            }

            $processed = $processed + 1
        } else {
            print $"⚠️  Warning: ($project) has no docs directory"
            $failed = $failed + 1
        }
    }

    # Generate main projects index
    print "\n📋 Generating projects overview..."
    generate_projects_index $source $projects

    # Validate links (optional, can be slow)
    # validate_links $source

    # Print summary
    print "\n📊 Update Summary:"
    print $"   ✅ Processed: ($processed)"
    print $"   ⚠️  Failed: ($failed)"
    print $"   📁 Total projects: ($projects | length)"

    print "\n✅ Documentation update complete!"
    print "💡 Run 'npx quartz build' to regenerate the site"
}

# Generate index.md for a project
def generate_project_index [project: string, docs_path: string] {
    let index_path = $"($docs_path)/index.md"

    # Try to get project info from git
    let repo_url = git -C $docs_path remote get-url origin
        | complete
        | if ($in.exit_code == 0) { $in.stdout | str trim } else { "" }

    let last_updated = git -C $docs_path log -1 --format="%ai"
        | complete
        | if ($in.exit_code == 0) { $in.stdout | str trim } else { "Unknown" }

    # Get list of markdown files
    let docs = ls $docs_path
        | where name =~ '\.md$'
        | get name
        | path basename
        | where {|name| $name != "index.md" }

    # Build documentation list
    let doc_list = if ($docs | length) > 0 {
        $docs | each {|doc|
            let name = ($doc | str replace '.md' '')
            let title = ($name | str replace '-' ' ' | str capitalize)
            $"- [[($name)|($title)]]"
        } | str join "\n"
    } else {
        "No documentation files found."
    }

    # Build repository line
    let repo_line = if ($repo_url != "") { $"**Repository**: ($repo_url)" } else { "" }

    let content = [$"---
title: ($project)
description: Documentation for ($project)
tags: [project, ($project)]
---

# ($project)

## Overview

Documentation for the ($project) project from raibid-labs.

($repo_line)
**Last Updated**: ($last_updated)

## Documentation

($doc_list)

## Contributing

For contribution guidelines, please refer to the main repository.

---

*This documentation is automatically aggregated from the project repository.*"] | str join "\n"

    $content | save --force $index_path
}

# Generate main projects index
def generate_projects_index [source: string, projects: list] {
    let index_path = $"($source)/index.md"

    # Build project list
    let project_list = $projects | each {|project|
        let title = ($project | str replace '-' ' ' | str capitalize)
        $"- [[($project)/index|($title)]]"
    } | str join "\n"

    # Get timestamp
    let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")

    let lines = [
        "---"
        "title: Projects"
        "description: Overview of all raibid-labs projects"
        "tags: [projects, overview]"
        "---"
        ""
        "# Raibid Labs Projects"
        ""
        "This section contains documentation aggregated from all active raibid-labs repositories."
        ""
        "## Active Projects"
        ""
        $project_list
        ""
        "## Navigation"
        ""
        "Use the sidebar to browse project documentation, or use the search feature to find specific topics."
        ""
        "## About This Documentation"
        ""
        "This documentation hub automatically aggregates content from all public raibid-labs repositories. Each project maintains its own documentation in its respective repository, and changes are synchronized daily."
        ""
        $"**Last Updated**: ($timestamp)"
        ""
        "---"
        ""
        "*For more information about raibid-labs, visit the [GitHub organization](https://github.com/raibid-labs).*"
    ]

    $lines | str join "\n" | save --force $index_path
}

# Validate internal links (optional)
def validate_links [source: string] {
    print "\n🔗 Validating internal links..."
    # Implementation for link validation
    # This is a placeholder for future enhancement
}
