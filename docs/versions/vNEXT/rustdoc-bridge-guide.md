---
title: Rust-Doc Bridge Plugin Guide
description: Bridge the gap between Obsidian markdown docs and Rust API documentation
tags: [documentation, plugin, rustdoc, wikilinks, api-reference]
---

# Rust-Doc Bridge Plugin Guide

## Overview

The **Rust-Doc Bridge** is a Quartz plugin that seamlessly bridges the gap between your Obsidian markdown documentation and Rust API documentation (rustdoc). Write `[[MyStruct]]` in your markdown files and have it automatically link to the correct API documentation.

## Problem Statement

When documenting Rust projects, you often have:

1. **Prose documentation** - High-level guides, tutorials, and explanations (in markdown)
2. **API documentation** - Detailed function/struct/trait docs generated from code comments (rustdoc)

These two worlds traditionally live separately:
- rustdoc is generated at `target/doc/crate_name/` 
- Markdown docs are in your project's `docs/` directory
- Linking between them requires manual URLs that break if the API structure changes

The Rust-Doc Bridge solves this by:
- Reading rustdoc JSON output
- Building an intelligent index of all API items
- Automatically resolving wikilinks (`[[MyStruct]]`) to the correct rustdoc URLs
- Supporting multiple naming conventions (PascalCase, snake_case, fully qualified paths)

## How It Works

### 1. Rust-Doc JSON Generation

During the documentation build process:

```bash
cargo doc --output-format json --no-deps
```

This generates a machine-readable JSON file containing:
- All crates, modules, functions, structs, traits, etc.
- Documentation text for each item
- The hierarchical structure and relationships

### 2. Index Building

The RustDocBridge plugin:
- Reads the generated `rustdoc.json`
- Builds an index mapping names → URLs
- Handles multiple naming conventions:
  - `[[MyStruct]]` → matches `MyStruct`
  - `[[my_function]]` → matches both `my_function` and `MyFunction`
  - `[[module::MyStruct]]` → matches fully qualified paths

### 3. Link Resolution

When processing your markdown:
- Finds all wikilinks (e.g., `[[MyStruct]]`)
- Checks if they match a rustdoc item
- Converts them to proper URLs pointing to the API docs
- Adds helpful metadata (item kind, crate name) as title attributes

## Usage

### Writing Wikilinks to API Items

In your markdown documentation, simply write wikilinks to API items:

```markdown
## Using Structs

The `[[Request]]` struct is used to build API requests. It implements
`[[Default]]` so you can construct one easily:

\`\`\`rust
let req = Request::default();
\`\`\`

For more details on request handling, see `[[RequestBuilder]]`.

## Error Handling

When errors occur, you'll get an `[[ApiError]]` result.
```

When built, these wikilinks automatically resolve to:
- `/api/my_crate/struct.Request.html`
- `/api/std/trait.Default.html`
- `/api/my_crate/struct.RequestBuilder.html`
- `/api/my_crate/enum.ApiError.html`

### Naming Conventions

The plugin is smart about naming conventions:

| Wikilink Format | Matches |
|---|---|
| `[[MyStruct]]` | struct `MyStruct` |
| `[[my_function]]` | function `my_function` OR `MyFunction` |
| `[[MyTrait]]` | trait `MyTrait` |
| `[[module::MyStruct]]` | `MyStruct` in `module` (fully qualified) |
| `[[Error]]` | enum `Error` or error type |

### Best Practices

1. **Use descriptive names**: `[[RequestBuilder]]` is better than `[[Builder]]`
2. **Qualify when needed**: Use `[[my_crate::MyStruct]]` for disambiguation
3. **Fallback to markdown links**: If a wikilink doesn't resolve, it stays as an internal link
4. **Keep wikilinks in backticks**: `` [[MyStruct]] `` in backticks reads naturally in markdown editors

## Configuration

### In quartz.config.ts

```typescript
Plugin.RustDocBridge({
  // Path to generated rustdoc.json
  rustdocJsonPath: "public/rustdoc.json",
  
  // Base URL for API docs
  apiDocsPath: "/api",
  
  // Optional: Which crates to include
  crateNames: ["my_crate", "my_utils"],
})
```

### Environment Setup

#### 1. Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

#### 2. Configure Your Rust Projects

Ensure each Rust project has proper documentation:

```rust
/// Brief description of the struct
/// 
/// Longer explanation goes here. You can use markdown:
/// - Lists
/// - **Bold** text
/// - [[Links to other items]]
pub struct MyStruct {
    /// Field documentation
    pub field: String,
}

/// A function that does something
/// 
/// # Examples
/// 
/// ```
/// let result = my_function(42);
/// ```
pub fn my_function(x: i32) -> Result<String, Error> {
    // ...
}
```

#### 3. Build Documentation

```bash
# Generate rustdoc JSON for all projects
nu scripts/build-rustdoc.nu --verbose

# Or manually in a specific project
cd projects/my_crate
cargo doc --output-format json --no-deps
```

## CI/CD Integration

The documentation hub automatically:

1. **Discovers** all Rust projects via the sync workflow
2. **Builds** rustdoc JSON for each project
3. **Indexes** all items into a unified JSON file
4. **Resolves** wikilinks during the Quartz build
5. **Deploys** the complete documentation site

No additional setup needed - it all happens in the GitHub Actions workflow.

## Advanced Usage

### Generating Documentation Stubs

You can optionally generate markdown "stub" pages for API items:

```bash
nu scripts/build-rustdoc.nu --generate-stubs --output-dir docs/api
```

This creates files like:
- `docs/api/request-struct.md`
- `docs/api/request-builder-struct.md`
- `docs/api/request-error-enum.md`

These can then be expanded with additional context and examples.

### Multi-Crate Workspaces

For Rust workspaces with multiple crates:

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crate-a", "crate-b", "crate-c"]

[workspace.metadata.docs]
all-features = true
```

The plugin automatically aggregates all workspace crates into a unified index.

### Filtering Items

To only index certain items, use a `.rustdocignore` file:

```
# Don't index private utilities
private::*
test_utils::*

# Only index public API
pub::**
```

## Troubleshooting

### Links Not Resolving

**Problem**: `[[MyStruct]]` doesn't link to the API docs

**Solutions**:
1. Check that the struct is `pub` (not private)
2. Verify the name matches exactly (case-sensitive by default)
3. Try fully qualified name: `[[crate_name::MyStruct]]`
4. Check `public/rustdoc.json` exists and is non-empty
5. Check the browser console for errors

### JSON Too Large

**Problem**: `rustdoc.json` is huge (>10MB)

**Solutions**:
1. Add `#![doc(no_crate_level_docs)]` to reduce documentation
2. Use feature flags to document only essential parts
3. Split large workspaces into smaller crates
4. Enable gzip compression in GitHub Pages

### Building Takes Too Long

**Problem**: `cargo doc` generation is slow

**Solutions**:
1. Use `--no-deps` to skip dependency docs
2. Parallelize across jobs: build each crate in a separate GitHub Actions job
3. Cache `target/` directory in CI
4. Use `--offline` flag if dependencies are cached

## Examples

### Example 1: Linking to a Request/Response Pattern

```markdown
# Building Requests

To construct an API request, use the `[[Request]]` builder pattern:

\`\`\`rust
let request = Request::builder()
    .method("POST")
    .uri("/api/users")
    .body(json!({"name": "Alice"}))?;
\`\`\`

The builder returns a `[[RequestResult]]` which is an alias for 
`Result<[[Request]], [[RequestError]]>`.

If construction fails, you'll get a `[[RequestError]]`. See the
[[Error Handling]] guide for details.
```

### Example 2: Documenting Traits

```markdown
# Custom Types

Our library provides several important traits:

- `[[Serializable]]` - Types that can be serialized to JSON
- `[[Deserializable]]` - Types that can be deserialized from JSON  
- `[[Transportable]]` - Types that can be sent over the wire

To implement one of these traits, see the examples in their 
documentation.
```

### Example 3: Cross-Project References

```markdown
# Integration with Authentication

The auth module provides the `[[auth::Token]]` struct which works
seamlessly with the `[[Request]]` builder:

\`\`\`rust
let token = auth::Token::from_string(auth_string);
let request = Request::builder()
    .token(token)
    .build()?;
\`\`\`
```

## Performance

- **Index Building**: ~1-5 seconds for typical projects
- **Link Resolution**: < 1ms per document (all items pre-indexed)
- **Cache**: Index is generated once per build, reused across all documents
- **Memory**: Typical JSON files are 1-5MB, minimal overhead

## Security

- The plugin is **read-only** - it doesn't modify rustdoc output
- No external network requests
- URLs are generated based on Quartz configuration
- JSON parsing is strict (errors are logged, not fatal)

## Future Enhancements

Planned features:

- [ ] Embedded API summaries (hover to see doc comments)
- [ ] Smart search across both prose and API docs
- [ ] Type signature visualization
- [ ] Dependency graph visualization
- [ ] Example extraction from doc comments
- [ ] Automatic backlinking from API docs to prose

## Contributing

The Rust-Doc Bridge plugin is maintained as part of the documentation hub. To contribute:

1. **Report Issues**: [GitHub Issues](https://github.com/raibid-labs/docs/issues)
2. **Suggest Features**: Use the discussions tab
3. **Submit PRs**: Fork and contribute improvements
4. **Plugin Code**: `quartz/plugins/transformers/rustdoc-bridge.ts`

## Related Resources

- [Quartz Plugin Documentation](https://quartz.jzhao.xyz/plugins)
- [Rust Documentation Guide](https://doc.rust-lang.org/rustdoc/)
- [cargo doc](https://doc.rust-lang.org/cargo/commands/cargo-doc.html)
- [Obsidian Wikilinks](https://help.obsidian.md/Linking+notes+and+files/Internal+links)

## License

MIT - Same as Quartz and the documentation hub.

---

**Need help?** Check the [[Troubleshooting]] section or open an issue.
