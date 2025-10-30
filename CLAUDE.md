# Claude Code Configuration - Raibid Labs Documentation Hub

## Project Overview

This repository serves dual purposes:
1. **Obsidian Vault**: For interactive note-taking and knowledge management
2. **Public Documentation Site**: Aggregates and publishes documentation from all raibid-labs public repositories

## Technology Stack

### Core Platform
- **Quartz v4**: Static site generator optimized for Obsidian vaults
  - Native Obsidian features (wikilinks, backlinks, graph view)
  - Built-in full-text search and graph visualization
  - Extensible plugin architecture
  - Perfect for AI-generated markdown content

### Aggregation System
- **Git Submodules**: Link to `/docs` directories from raibid-labs repositories
- **Nushell Scripts**: Automate discovery, filtering, and synchronization
- **GitHub Actions**: Continuous integration and deployment

## Repository Structure

```
docs/
├── index.md                    # Homepage
├── content/                    # Main documentation content
│   ├── projects/              # Aggregated from submodules
│   │   ├── project-1/        # Submodule: raibid-labs/project-1/docs
│   │   ├── project-2/        # Submodule: raibid-labs/project-2/docs
│   │   └── ...
│   └── guides/               # Local documentation
├── scripts/                   # Nushell automation scripts
│   ├── discover-repos.nu     # GitHub API repo discovery
│   ├── sync-submodules.nu    # Submodule management
│   ├── update-docs.nu        # Documentation updates
│   └── build-site.nu         # Build orchestration
├── config/                    # Configuration files
│   ├── ignorelist.json       # Repositories to exclude
│   └── quartz.config.ts      # Quartz configuration
├── .github/
│   └── workflows/
│       └── sync-and-deploy.yml  # Automated sync & deploy
└── CLAUDE.md                 # This file
```

## Key Features

### 1. Automated Repository Discovery
- Nushell script queries GitHub API for raibid-labs org repositories
- Filters based on ignorelist (forked repos, private repos, etc.)
- Identifies public repos with `/docs` directories

### 2. Submodule Management
- Automatically adds/updates submodules for new repositories
- Maintains symlinks or copies docs into Quartz content directory
- Handles repository renames and deletions gracefully

### 3. Obsidian Integration
- Full compatibility with Obsidian vault features
- Graph view shows relationships between docs
- Wikilinks work across aggregated documentation
- Local editing with instant preview

### 4. Publication Pipeline
- GitHub Actions trigger on schedule (daily) or manual dispatch
- Syncs latest docs from all tracked repositories
- Builds static site with Quartz
- Deploys to GitHub Pages (or other hosting)

## Configuration Files

### ignorelist.json
```json
{
  "repositories": [
    "forked-repo-name",
    "private-repo-name",
    "archived-project"
  ],
  "patterns": [
    "fork-*",
    "archive-*"
  ]
}
```

### Quartz Configuration
Located in `quartz.config.ts`, customized for:
- Multi-source content aggregation
- Custom navigation structure
- Theme optimized for technical documentation
- Search configuration

## Scripts Overview

### discover-repos.nu
Queries GitHub API to find all public raibid-labs repositories, filters against ignorelist, outputs list of repositories with docs directories.

### sync-submodules.nu
Compares discovered repos with current submodules, adds new submodules for repositories not yet tracked, updates existing submodules to latest commits, removes submodules for deleted/ignored repositories.

### update-docs.nu
Updates all submodules to latest main/master branch, copies/symlinks docs into Quartz content directory, generates index files and navigation structure.

### build-site.nu
Orchestrates full build pipeline: sync submodules, update docs, run Quartz build, validate output.

## Workflow

### Daily Automated Sync
1. GitHub Action triggers on schedule
2. `discover-repos.nu` finds new/changed repositories
3. `sync-submodules.nu` updates submodule configuration
4. `update-docs.nu` pulls latest documentation
5. `build-site.nu` generates static site
6. Deploy to GitHub Pages

### Manual Development
1. Edit documentation locally in Obsidian
2. Run `npx quartz build --serve` for live preview
3. Commit changes and push
4. CI/CD pipeline handles deployment

## Requirements

### Prerequisites
- Node.js v22+ and npm v10.9.2+
- Nushell (latest stable version)
- Git with submodule support
- GitHub CLI (gh) for API access
- Obsidian (optional, for local editing)

### Environment Variables
- `GITHUB_TOKEN`: Personal access token with `repo` and `read:org` scopes
- `GITHUB_ORG`: Set to `raibid-labs`

## Getting Started

### Initial Setup
```bash
# Clone repository
git clone https://github.com/raibid-labs/docs.git
cd docs

# Install Quartz
npx quartz create

# Install dependencies
npm install

# Discover and sync repositories
nu scripts/discover-repos.nu
nu scripts/sync-submodules.nu

# Build and preview
npx quartz build --serve
```

### Adding to Ignorelist
Edit `config/ignorelist.json` and add repository name or pattern.

### Manual Sync
```bash
nu scripts/update-docs.nu
```

## Development Guidelines

### Markdown Standards
- Use CommonMark-compliant markdown
- Obsidian-style wikilinks for internal references: `[[page-name]]`
- Front matter for metadata:
  ```yaml
  ---
  title: Page Title
  description: Brief description
  tags: [tag1, tag2]
  ---
  ```

### File Organization
- Keep aggregated content under `content/projects/`
- Local documentation goes in `content/guides/`
- Use clear, descriptive filenames
- Maintain consistent directory structure

### Scripting Guidelines
- All automation in Nushell for consistency
- Scripts should be idempotent (safe to run multiple times)
- Include error handling and logging
- Document script parameters and usage

## Quartz Features

### Built-in Components
- **Graph View**: Visual map of document relationships
- **Backlinks**: Automatic bidirectional links
- **Table of Contents**: Auto-generated from headings
- **Search**: Full-text search with Pagefind
- **Dark Mode**: Toggle between light/dark themes

### Customization
- Edit `quartz.config.ts` for site-wide settings
- Custom components in `quartz/components/`
- Styling via CSS in `quartz/styles/`
- Plugins in `quartz/plugins/`

## CI/CD Pipeline

### GitHub Actions Workflow
```yaml
# Triggers:
- schedule: Daily at 02:00 UTC
- workflow_dispatch: Manual trigger
- push: On main branch changes

# Steps:
1. Checkout repository with submodules
2. Setup Node.js and Nushell
3. Authenticate with GitHub
4. Run discovery and sync scripts
5. Build Quartz site
6. Deploy to GitHub Pages
```

## Best Practices

### Documentation Standards
1. **Consistency**: Follow organization-wide style guide
2. **Accessibility**: Use semantic HTML and ARIA labels
3. **SEO**: Meaningful titles, descriptions, and headings
4. **Navigation**: Clear hierarchy and breadcrumbs
5. **Search**: Use descriptive headings for better findability

### Repository Hygiene
1. Regular submodule updates to stay current
2. Monitor for broken links across repositories
3. Archive old/deprecated content appropriately
4. Tag releases for version-specific documentation

### Performance
1. Optimize images before committing
2. Use lazy loading for heavy content
3. Minimize custom JavaScript
4. Leverage Quartz's built-in optimizations

## Troubleshooting

### Submodule Issues
```bash
# Reset submodules
git submodule deinit -f .
git submodule update --init --recursive

# Update specific submodule
git submodule update --remote content/projects/project-name
```

### Build Failures
```bash
# Clean build
rm -rf .quartz-cache public
npx quartz build

# Check Node version
node --version  # Should be v22+
```

### GitHub API Rate Limits
- Use authenticated requests (GITHUB_TOKEN)
- Implement exponential backoff in scripts
- Cache API responses where possible

## Resources

### Documentation
- [Quartz Documentation](https://quartz.jzhao.xyz/)
- [Obsidian Documentation](https://help.obsidian.md/)
- [Nushell Book](https://www.nushell.sh/book/)
- [GitHub API Documentation](https://docs.github.com/en/rest)

### Community
- [Quartz Discord](https://discord.gg/cRFFHYye7t)
- [Obsidian Forum](https://forum.obsidian.md/)
- raibid-labs GitHub Organization

## Maintenance

### Regular Tasks
- **Daily**: Automated sync via GitHub Actions
- **Weekly**: Review new repositories for inclusion
- **Monthly**: Update ignorelist, prune archived projects
- **Quarterly**: Audit documentation quality and coverage

### Version Updates
- Keep Quartz updated to latest stable version
- Monitor for Nushell breaking changes
- Update Node.js as required by Quartz

## License

This documentation hub aggregates content from multiple repositories. Each project retains its original license. See individual project directories for specific licensing information.

---

**Note**: This repository is public and only aggregates from public repositories within the raibid-labs organization. Ensure no sensitive information is committed.
