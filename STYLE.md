# Documentation Style Guide

This style guide defines the writing conventions, formatting standards, and voice for all documentation in the raibid-labs/docs repository.

## Table of Contents

- [Writing Voice](#writing-voice)
- [Formatting](#formatting)
- [Code Examples](#code-examples)
- [Markdown Conventions](#markdown-conventions)
- [Front Matter](#front-matter)
- [Links and References](#links-and-references)
- [Common Pitfalls](#common-pitfalls)

## Writing Voice

### Tone and Style

**Be Clear and Concise**
- Prefer simple, direct language over complex terminology
- Use short sentences and paragraphs
- Get to the point quickly

**Use Active Voice**
- ✅ "Run the command to install dependencies"
- ❌ "Dependencies should be installed by running the command"

**Use Present Tense**
- ✅ "The script creates a new directory"
- ❌ "The script will create a new directory"

**Address the Reader Directly (Second Person)**
- ✅ "You can configure the settings in the config file"
- ❌ "One can configure the settings" or "Users can configure"

**Be Helpful, Not Condescending**
- ✅ "This guide walks you through the setup process"
- ❌ "Simply follow these basic steps" (avoid "simply", "just", "obviously")

### Technical Accuracy

- **Be precise** - Use exact command names, file paths, and terminology
- **Be consistent** - Use the same terms for the same concepts throughout
- **Be current** - Keep documentation up-to-date with latest versions

### Accessibility

- **Use descriptive headings** - Help readers scan and navigate
- **Include alt text** for images
- **Avoid jargon** or explain it when necessary
- **Provide context** - Don't assume prior knowledge

## Formatting

### Headings

Use ATX-style headings (# symbols):

```markdown
# H1 - Page Title (only one per document)
## H2 - Major Section
### H3 - Subsection
#### H4 - Minor Subsection
```

**Rules:**
- Only one H1 per document (the page title)
- Use sentence case, not Title Case
- Don't skip heading levels (H2 → H4)
- Keep headings descriptive and concise

### Emphasis

**Bold** for UI elements and important terms:
```markdown
Click the **Save** button.
The **package.json** file contains dependencies.
```

*Italic* for emphasis and new terms:
```markdown
This is *required* for the feature to work.
The *manifest file* defines available versions.
```

**Code formatting** for code, commands, and file paths:
```markdown
Run `npm install` to install dependencies.
Edit the `config/settings.json` file.
The `process.env.NODE_ENV` variable controls the environment.
```

### Lists

**Unordered Lists** for items without sequence:
```markdown
- First item
- Second item
- Third item
  - Nested item
  - Another nested item
```

**Ordered Lists** for sequential steps:
```markdown
1. First step
2. Second step
3. Third step
   1. Substep
   2. Another substep
```

**Task Lists** for checklists:
```markdown
- [ ] Incomplete task
- [x] Completed task
```

### Tables

Use tables for structured data:

```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

**Guidelines:**
- Keep tables simple (max 4-5 columns)
- Use descriptive headers
- Align text consistently

## Code Examples

### Code Fences

Always use fenced code blocks with language specification:

````markdown
```bash
npm install
npm run dev
```

```typescript
export function hello(): string {
  return "Hello, World!";
}
```

```json
{
  "name": "example",
  "version": "1.0.0"
}
```
````

### Supported Languages

Common language identifiers:
- `bash` or `sh` - Shell commands
- `typescript` or `ts` - TypeScript
- `javascript` or `js` - JavaScript
- `json` - JSON data
- `yaml` - YAML configuration
- `markdown` or `md` - Markdown
- `rust` - Rust code
- `python` - Python code
- `toml` - TOML configuration

### Command Examples

**Shell Commands:**
```markdown
```bash
# Comment explaining the command
npm install --save-dev typescript

# Multiple related commands
npm run build
npm test
```
```

**Include Comments:**
- Explain what the command does
- Highlight important flags or options
- Warn about destructive operations

**Show Output When Helpful:**
````markdown
```bash
$ npm --version
10.9.2
```
````

### Code Snippets

**Keep snippets focused:**
- Show only relevant code
- Use `...` or comments to indicate omitted code
- Highlight the important parts

**Example:**
```typescript
// ... previous code

export function example() {
  // This is the important part
  const result = processData();
  return result;
}

// ... more code
```

### Inline Code

Use backticks for:
- Command names: `npm`, `git`, `cargo`
- File names: `package.json`, `README.md`
- Function names: `processData()`, `main()`
- Variable names: `NODE_ENV`, `API_KEY`
- Short code snippets: `const x = 5;`

## Markdown Conventions

### Line Length

- Prefer 80-100 characters per line for readability
- Break long lines at natural points (punctuation, conjunctions)
- Code blocks can exceed line length if necessary

### Blank Lines

- One blank line between paragraphs
- One blank line before and after headings
- One blank line before and after code blocks
- One blank line before and after lists

### Indentation

- Use 2 spaces for nested lists
- Use 4 spaces for nested code in lists
- Use spaces, not tabs (except in code blocks where appropriate)

### Special Elements

**Callouts/Admonitions:**
```markdown
> **Note:** This is important information.

> **Warning:** This operation is destructive.

> **Tip:** Here's a helpful hint.
```

**Blockquotes:**
```markdown
> This is a quote from documentation or external source.
> - Author Name
```

**Horizontal Rules:**
```markdown
Use three dashes for horizontal separators:

---

Next section begins here.
```

## Front Matter

Include YAML front matter at the top of markdown files:

```yaml
---
title: "Page Title"
description: "Brief description for search and previews"
tags:
  - tag1
  - tag2
  - tag3
date: 2025-12-05
author: "Author Name"
---
```

**Required Fields:**
- `title` - Page title (string)
- `description` - Brief summary (string, 50-160 characters)

**Optional Fields:**
- `tags` - Array of tags for categorization
- `date` - Publication date (YYYY-MM-DD format)
- `author` - Author name or GitHub username
- `draft` - Set to `true` to exclude from build

## Links and References

### Internal Links

**Absolute paths** for cross-version links:
```markdown
[See the guide](/docs/versions/vNEXT/guide.md)
```

**Relative paths** for same-directory links:
```markdown
[Related topic](./related-topic.md)
```

### External Links

Always include link text and URL:
```markdown
[Quartz Documentation](https://quartz.jzhao.xyz/)
```

**Guidelines:**
- Use descriptive link text (not "click here")
- Include protocol (https://)
- Verify links work before committing

### Reference Links

For repeated links, use reference style:
```markdown
Check the [Quartz docs][quartz] for details.
See the [GitHub guide][gh-guide] for more info.

[quartz]: https://quartz.jzhao.xyz/
[gh-guide]: https://docs.github.com/
```

### Anchor Links

Link to sections within the same page:
```markdown
See [Code Examples](#code-examples) above.
Jump to [Configuration](#configuration) section.
```

## Common Pitfalls

### Avoid These Patterns

**Don't use vague language:**
- ❌ "Kind of like", "sort of", "basically"
- ✅ Use precise descriptions

**Don't assume knowledge:**
- ❌ "As you know", "obviously", "of course"
- ✅ Provide context or link to background

**Don't be condescending:**
- ❌ "Simply", "just", "trivial", "easy"
- ✅ Be neutral and helpful

**Don't use future tense unnecessarily:**
- ❌ "The next section will explain"
- ✅ "The next section explains"

**Don't mix code styles:**
- ❌ Some commands with `$` prompt, others without
- ✅ Be consistent throughout document

### Grammar and Punctuation

**Use Oxford commas:**
- ✅ "Apples, oranges, and bananas"
- ❌ "Apples, oranges and bananas"

**Consistent punctuation in lists:**
- If list items are complete sentences, use periods
- If list items are fragments, no periods
- Be consistent within each list

**Code in sentences:**
- Use backticks: "The `npm install` command downloads packages."
- Treat as singular: "The `package.json` file defines dependencies."

## File Naming

**Markdown files:**
- Lowercase with hyphens: `getting-started.md`
- Descriptive names: `api-reference.md`, not `api.md`
- Avoid version numbers in names: `guide.md`, not `guide-v1.md`

**Images and assets:**
- Lowercase with hyphens: `architecture-diagram.png`
- Include descriptive names
- Store in appropriate directories

## Version Control

**Commit messages:**
Follow conventional commit format:
```
feat: Add installation guide for macOS
fix: Correct broken link in API docs
docs: Update contributing guidelines
style: Format code examples consistently
```

**Line endings:**
- Use LF (Unix-style) line endings
- Configure `.gitattributes` appropriately

## Review Checklist

Before submitting documentation:

- [ ] Spelling and grammar checked
- [ ] Code examples tested
- [ ] Links verified (no broken links)
- [ ] Front matter included
- [ ] Headings follow hierarchy
- [ ] Voice and tone consistent
- [ ] Images have alt text
- [ ] File named appropriately
- [ ] Follows versioning policy

## Questions?

For questions about this style guide:
- Open an issue with the `documentation` label
- See [CONTRIBUTING.md](./docs/CONTRIBUTING.md)
- Reference the [Quartz documentation](https://quartz.jzhao.xyz/)

---

**Maintained by**: raibid-labs
**Last updated**: 2025-12-05
