# Raibid Labs Documentation Hub

Centralized documentation aggregating content from all public raibid-labs repositories using Quartz v4 and automated Nushell scripts.

🌐 **Live Site:** https://raibid-labs.github.io/docs

## Overview

This repository serves dual purposes:
1. **Obsidian Vault** - Local knowledge management for editing documentation
2. **Static Website** - Published documentation site using Quartz v4

Documentation is automatically aggregated from all public raibid-labs repositories that contain a `/docs` directory via git submodules.

## Features

- ✅ Automatic repository discovery via GitHub API
- ✅ Git submodules for multi-repository aggregation
- ✅ Automated synchronization with daily GitHub Actions
- ✅ Full-text search and graph visualization
- ✅ Obsidian-compatible (wikilinks, backlinks, graph view)
- ✅ Configurable ignorelist for repository exclusion
- ✅ All automation written in Nushell

## Quick Start

### Prerequisites

- **Node.js** >= 22.0.0
- **npm** >= 10.9.2
- **Nushell** >= 0.105.0
- **GitHub CLI** (`gh`) - [Installation guide](https://cli.github.com/)

### Installation

```bash
# Clone the repository
git clone --recursive https://github.com/raibid-labs/docs.git
cd docs

# Install dependencies
npm install

# Authenticate GitHub CLI (if not already done)
gh auth login
```

### Local Development

```bash
# Build and serve the site locally
npm run dev

# Or use Quartz directly
npx quartz build --serve
```

Visit http://localhost:8080 to view the site.

### Running Scripts

```bash
# Discover repositories from raibid-labs organization
npm run discover

# Sync git submodules
npm run sync

# Update documentation and generate indices
nu scripts/update-docs.nu --generate-index --verbose

# Full build pipeline
nu scripts/build-site.nu --verbose
```

## Architecture

### Directory Structure

```
docs/
├── config/
│   ├── ignorelist.json          # Repository exclusion configuration
│   └── ignorelist-internal.json # Template for internal docs repo
├── docs/
│   ├── content/
│   │   ├── projects/            # Project submodules (auto-generated)
│   │   └── guides/              # Documentation guides
│   └── index.md                 # Homepage
├── scripts/
│   ├── discover-repos.nu        # GitHub API repository discovery
│   ├── sync-submodules.nu       # Git submodule management
│   ├── update-docs.nu           # Documentation updates and indexing
│   └── build-site.nu            # Complete build pipeline
├── .github/
│   └── workflows/
│       └── sync-and-deploy.yml  # CI/CD automation
├── quartz.config.ts             # Quartz configuration
├── package.json                 # Node.js dependencies
└── CLAUDE.md                    # Claude Code configuration
```

### Automation Workflow

1. **Repository Discovery** (`discover-repos.nu`)
   - Queries GitHub API for raibid-labs organization repos
   - Filters based on ignorelist configuration
   - Checks for `/docs` directory presence
   - Outputs `discovered-repos.json`

2. **Submodule Synchronization** (`sync-submodules.nu`)
   - Adds new repository docs as submodules
   - Updates existing submodules
   - Removes stale submodules
   - Maintains `.gitmodules` file

3. **Documentation Updates** (`update-docs.nu`)
   - Pulls latest changes from all submodules
   - Generates index files for navigation
   - Validates markdown files
   - Creates project overview pages

4. **Site Build** (`build-site.nu`)
   - Orchestrates full pipeline
   - Runs all scripts in sequence
   - Builds static site with Quartz
   - Optional local server for preview

### GitHub Actions

Automated workflow runs:
- **Daily at 2 AM UTC** - Scheduled sync
- **On push to main** - When workflow or scripts change
- **Manual trigger** - Via workflow_dispatch

## Configuration

### Ignorelist

Edit `config/ignorelist.json` to control which repositories are included:

```json
{
  "repositories": ["docs"],           // Explicitly excluded repos
  "patterns": ["fork-*", "archive-*"], // Regex patterns to exclude
  "exclude_forks": true,                // Exclude forked repositories
  "exclude_archived": true,             // Exclude archived repositories
  "exclude_private": true,              // Exclude private repositories (public repo only)
  "require_docs_directory": true        // Only include repos with /docs folder
}
```

### Quartz Configuration

Edit `quartz.config.ts` to customize:
- Site title and branding
- Theme and colors
- Enabled plugins
- Ignore patterns
- Base URL

## Private Documentation

For aggregating private repository documentation, see the two-repo approach:

📚 **[Private Docs Setup Guide](./docs/content/guides/private-docs-setup.md)**

### Quick Summary

**Public Docs (this repo):**
- Repository: `raibid-labs/docs` (public)
- Aggregates: Public repos only
- Deployment: GitHub Pages
- URL: https://raibid-labs.github.io/docs

**Internal Docs (companion repo):**
- Repository: `raibid-labs/docs-internal` (private)
- Aggregates: All repos (public + private)
- Deployment: Local builds, Cloudflare Pages, or artifacts
- Access: Organization members only

Both repos share the same automation scripts and tooling for consistency.

## Usage with Obsidian

This repository can be opened as an Obsidian vault:

1. Install [Obsidian](https://obsidian.md/)
2. Open this repository as a vault
3. Enable recommended plugins:
   - Wikilinks
   - Graph View
   - Backlinks
   - Search

The site will automatically render Obsidian features:
- `[[wikilinks]]`
- Tags with `#tag`
- Backlinks and graph relationships
- Callouts and admonitions

See [Obsidian Usage Guide](./docs/content/guides/obsidian-usage.md) for details.

## Deployment

### GitHub Pages (Enabled)

Site is automatically deployed to GitHub Pages via the gh-pages branch:
- **URL:** https://raibid-labs.github.io/docs
- **Branch:** gh-pages
- **Trigger:** Automatic on successful workflow runs

### Manual Deployment

```bash
# Build the site
npx quartz build

# Deploy to gh-pages branch
cd public
git init
git add -A
git commit -m "Deploy documentation"
git push -f https://github.com/raibid-labs/docs.git HEAD:gh-pages
```

## Troubleshooting

### Submodule Issues

```bash
# Initialize all submodules
git submodule update --init --recursive

# Update all submodules to latest
git submodule update --remote --merge

# Remove a problematic submodule
git submodule deinit -f docs/content/projects/repo-name
git rm -f docs/content/projects/repo-name
rm -rf .git/modules/docs/content/projects/repo-name
```

### Build Errors

```bash
# Clean build cache
rm -rf public .quartz-cache

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Verify Nushell syntax
nu --version  # Should be >= 0.105.0
nu scripts/discover-repos.nu --help
```

### GitHub API Rate Limits

```bash
# Check rate limit status
gh api rate_limit

# Authenticate with personal access token
gh auth login --with-token < token.txt
```

## Contributing

### Adding Documentation to Your Repo

To have your repository's documentation included in this hub:

1. Create a `/docs` directory in your repository
2. Add markdown files to `/docs`
3. Ensure your repository is:
   - Public (for this docs repo)
   - Not in the ignorelist
   - Part of the raibid-labs organization
4. Wait for the next scheduled sync (daily at 2 AM UTC) or trigger manually

### Manual Sync Trigger

Organization members can manually trigger synchronization:

```bash
gh workflow run sync-and-deploy.yml --repo raibid-labs/docs
```

Or via the GitHub UI:
1. Go to Actions tab
2. Select "Sync Documentation and Deploy"
3. Click "Run workflow"

## Technology Stack

- **[Quartz v4](https://quartz.jzhao.xyz/)** - Static site generator
- **[Nushell](https://www.nushell.sh/)** - Automation scripting
- **[GitHub Actions](https://github.com/features/actions)** - CI/CD
- **[GitHub Pages](https://pages.github.com/)** - Hosting
- **[Obsidian](https://obsidian.md/)** - Local vault editing (optional)
- **Node.js** - Build toolchain
- **Git Submodules** - Multi-repository aggregation

## Project Status

✅ **Active Development**

This documentation hub is actively maintained and automatically updated daily.

- Last deployed: Check [Actions tab](https://github.com/raibid-labs/docs/actions)
- Live site: https://raibid-labs.github.io/docs
- Issues: [GitHub Issues](https://github.com/raibid-labs/docs/issues)

## License

Documentation content: Various (see individual repositories)

Quartz v4: MIT License (see Quartz documentation)

Automation scripts: MIT License

---

**Maintained by**: raibid-labs
**Questions?** Open an issue or check the [guides](./docs/content/guides/).
