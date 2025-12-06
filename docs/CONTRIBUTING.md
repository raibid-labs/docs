# Contributing to Raibid Labs Documentation

Thank you for your interest in contributing to the Raibid Labs documentation hub! This guide will help you understand how to contribute effectively.

## Quick Links

- [Documentation Structure](./STRUCTURE.md) - Organization and versioning policy
- [Style Guide](../STYLE.md) - Writing style and formatting conventions
- [Code of Conduct](../CODE_OF_CONDUCT.md) - Community guidelines

## Ways to Contribute

### 1. Adding Documentation from Your Repository

If you maintain a repository in the raibid-labs organization:

1. **Create a `/docs` directory** in your repository
2. **Add markdown files** with clear, user-facing documentation
3. **Ensure your repository is**:
   - Public (for the public docs hub)
   - Not in the ignorelist (`config/ignorelist.json`)
   - Part of the raibid-labs organization
4. **Wait for automatic sync** (daily at 2 AM UTC) or trigger manually

Your documentation will be automatically aggregated via git submodules.

### 2. Improving Existing Documentation

To improve documentation in this repository:

1. **Fork the repository** or create a branch
2. **Make your changes** in the appropriate version directory (`docs/versions/vNEXT/`)
3. **Follow the style guide** (see `STYLE.md`)
4. **Test locally**:
   ```bash
   npm install
   npm run dev
   ```
5. **Submit a pull request**

### 3. Reporting Issues

Found a problem? Open an issue:

- **Broken links** - Use issue label: `broken-link`
- **Outdated content** - Use issue label: `documentation`
- **Missing documentation** - Use issue label: `enhancement`
- **Typos/errors** - Use issue label: `bug`

## Documentation Standards

### Version Placement

- **New documentation** → `docs/versions/vNEXT/`
- **Bug fixes to released docs** → Specific version directory (e.g., `docs/versions/v1.0.0/`)
- **Never** add user-facing docs to the root `docs/` directory (except governance docs)

### Required Elements

Every documentation file should include:

1. **Clear title** (H1 heading)
2. **Introduction** - What this document covers
3. **Prerequisites** (if applicable)
4. **Step-by-step content**
5. **Examples** (code samples, screenshots)
6. **Related links** (see also section)

### File Naming

- Use lowercase with hyphens: `getting-started.md`, `api-reference.md`
- Be descriptive: `quartz-setup.md` not `setup.md`
- Avoid version numbers in filenames (use directory structure)

### Front Matter

Include YAML front matter for better organization:

```yaml
---
title: "Getting Started Guide"
description: "Learn how to set up and use the documentation hub"
tags:
  - guide
  - setup
  - beginner
date: 2025-12-05
---
```

## Style Guidelines

### Writing Style

- **Clear and concise** - Prefer simple language over jargon
- **Active voice** - "Run the command" not "The command should be run"
- **Present tense** - "The script creates" not "The script will create"
- **Second person** - "You can configure" not "One can configure"

### Code Examples

Use fenced code blocks with language specification:

````markdown
```bash
npm install
npm run dev
```

```typescript
export function example() {
  console.log("Hello");
}
```
````

### Links

- **Absolute paths** for internal links: `/docs/versions/vNEXT/guide.md`
- **Relative paths** for same-directory links: `./guide.md`
- **External links** should include description: `[Quartz Documentation](https://quartz.jzhao.xyz/)`

### Images

- Store images in `content/assets/` or version-specific directories
- Use descriptive alt text: `![Architecture diagram showing component relationships](./diagram.png)`
- Optimize images (compress, reasonable dimensions)

## Pull Request Process

### Before Submitting

1. **Read the documentation** you're changing
2. **Run the linter**: `npm run lint` (if available)
3. **Test locally**: `npm run dev`
4. **Check for broken links**
5. **Review your changes**

### PR Guidelines

1. **Create a descriptive title**:
   - Good: "Add installation guide for Windows"
   - Bad: "Update docs"

2. **Write a clear description**:
   - What changed?
   - Why is this change needed?
   - Related issues (if any)

3. **Keep PRs focused**:
   - One topic per PR
   - Don't mix unrelated changes

4. **Respond to feedback**:
   - Address reviewer comments
   - Update the PR as needed

### PR Template

```markdown
## Summary
Brief description of changes

## Changes Made
- Added X documentation
- Updated Y section
- Fixed Z broken links

## Testing
- [ ] Built site locally (`npm run dev`)
- [ ] Verified links work
- [ ] Checked formatting
- [ ] Reviewed in browser

## Related Issues
Closes #123
```

## Development Workflow

### Local Setup

```bash
# Clone with submodules
git clone --recursive https://github.com/raibid-labs/docs.git
cd docs

# Install dependencies
npm install

# Authenticate GitHub CLI (for scripts)
gh auth login
```

### Common Tasks

```bash
# Start local development server
npm run dev

# Discover repositories
npm run discover

# Sync submodules
npm run sync

# Build site
npm run build
```

### Branch Naming

Use descriptive branch names:

- Feature: `feature/add-api-docs`
- Fix: `fix/broken-link-in-guide`
- Documentation: `docs/update-contributing-guide`
- Issue: `issue-5-documentation-cleanup`

## CI/CD Requirements

All PRs must pass:

1. **Documentation linting** - Markdown style checks
2. **Link validation** - No broken links
3. **Front-matter validation** - Required metadata present
4. **Build test** - Site builds successfully

See `.github/workflows/docs-lint.yml` for details.

## Versioning and Releases

### During Development

- All changes go to `docs/versions/vNEXT/`
- No changes to released versions (except critical fixes)

### During Release

- `vNEXT` becomes versioned (e.g., `v1.0.0`)
- New `vNEXT` directory created
- `versions.json` updated
- `latest` symlink updated
- Release notes generated

See [RELEASE.md](./RELEASE.md) for full release process.

## Code Review

### For Contributors

- Be open to feedback
- Ask questions if unclear
- Make requested changes promptly
- Thank reviewers for their time

### For Reviewers

- Be respectful and constructive
- Explain the "why" behind suggestions
- Approve when standards are met
- Use GitHub's review features

## Getting Help

### Questions?

- **Documentation questions** - Open an issue with `question` label
- **Technical issues** - Open an issue with `bug` label
- **Feature requests** - Open an issue with `enhancement` label

### Resources

- [Quartz Documentation](https://quartz.jzhao.xyz/)
- [Obsidian Documentation](https://help.obsidian.md/)
- [GitHub Markdown Guide](https://guides.github.com/features/mastering-markdown/)
- [Nushell Documentation](https://www.nushell.sh/)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT License for automation scripts).

Documentation content may have various licenses depending on the source repository.

## Recognition

Contributors are recognized:
- In git history
- In release notes
- In the project README (for significant contributions)

Thank you for contributing to Raibid Labs documentation!

---

**Questions?** Open an issue or reach out to the maintainers.
