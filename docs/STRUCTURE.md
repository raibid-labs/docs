# Documentation Structure

This document describes the required structure and organization of the raibid-labs/docs repository.

## Directory Layout

```
docs/
├── versions/              # Versioned documentation
│   ├── vNEXT/            # Next release (unreleased changes)
│   ├── v1.0.0/           # Released version 1.0.0
│   ├── v1.1.0/           # Released version 1.1.0
│   └── latest -> v1.1.0  # Symlink to latest stable version
├── versions.json         # Version manifest
├── STRUCTURE.md          # This file
├── CONTRIBUTING.md       # Contribution guidelines
└── RELEASE.md            # Release process documentation

content/
├── index.md              # Landing page (required)
├── guides/               # User guides
├── projects/             # Project submodules (auto-generated)
└── blog/                 # Blog posts and updates
```

## Versioning Policy

### Version Directory Structure

Each version directory should contain:

1. **Introduction** - Overview and what's new in this version
2. **Installation** - Setup and prerequisites
3. **Usage** - Getting started guides and tutorials
4. **API Reference** - Technical documentation and API docs
5. **Changelog** - Version-specific changes (or link to main CHANGELOG.md)

### Version Lifecycle

1. **vNEXT** - Development version containing unreleased changes
   - All new documentation goes here first
   - Updated continuously during development
   - Becomes a versioned release during release process

2. **Released Versions** (e.g., v1.0.0, v1.1.0)
   - Frozen snapshots of documentation at release time
   - Only receive critical fixes (typos, broken links)
   - Never receive feature documentation updates

3. **latest** - Symlink to the most recent stable version
   - Always points to the highest version number
   - Updated during release process

### versions.json Manifest

The `versions.json` file tracks all available versions:

```json
{
  "latest": "v1.1.0",
  "versions": [
    {
      "name": "v1.1.0",
      "released": "2025-01-15",
      "status": "stable"
    },
    {
      "name": "v1.0.0",
      "released": "2024-12-01",
      "status": "stable"
    },
    {
      "name": "vNEXT",
      "released": null,
      "status": "development"
    }
  ]
}
```

## Required Sections

Every version directory must contain:

### 1. Introduction
- What is this project/release?
- Key features and capabilities
- Target audience
- What's new in this version

### 2. Installation
- System requirements
- Prerequisites
- Step-by-step installation instructions
- Verification steps

### 3. Usage
- Quick start guide
- Common use cases
- Example workflows
- Best practices

### 4. API Reference
- Technical specifications
- Function/method documentation
- Configuration options
- Advanced features

### 5. Changelog Links
- Link to version-specific changes
- Migration guides (for major versions)
- Breaking changes

## Prohibited Patterns

### No Archive Directories
- **Forbidden**: `archive/`, `old/`, `deprecated/`, `backup/`
- **Reason**: Use git history for old content
- **Alternative**: Remove outdated docs or update them in-place

### No Unversioned Documentation
- **Forbidden**: Documentation in `docs/` root (except meta-docs)
- **Reason**: All user-facing docs must be versioned
- **Exception**: Repository governance docs (STRUCTURE.md, CONTRIBUTING.md, RELEASE.md)

### Landing Page Required
- Every version must have clear entry point
- Link to version selector
- Navigation to key sections

## Content Guidelines

### User-Facing Documentation Only
- Focus on what users need to know
- Remove internal notes, AI conversation logs, implementation details
- Keep development notes in separate repository or private docs

### Distilled Content
- Prefer concise, actionable guides over verbose explanations
- Use code examples and practical demonstrations
- Link to external resources instead of duplicating content

### Consistent Structure
- Use the same section organization across versions
- Maintain similar file naming conventions
- Keep front-matter metadata consistent

## Validation

Documentation structure is validated by CI:
- Check for required sections
- Validate `versions.json` format
- Verify no archive directories exist
- Ensure landing page exists
- Check for broken links
- Validate front-matter metadata

See `.github/workflows/docs-lint.yml` for implementation details.

## Migration from Unversioned Docs

When migrating existing documentation:

1. Create `docs/versions/vNEXT/`
2. Move all user-facing docs into vNEXT
3. Delete AI dumps, conversation logs, internal notes
4. Create `versions.json` manifest
5. Update README to reference versioned docs
6. Ensure all links point to versioned paths

## Questions?

See [CONTRIBUTING.md](./CONTRIBUTING.md) for contribution guidelines or open an issue for clarification.
