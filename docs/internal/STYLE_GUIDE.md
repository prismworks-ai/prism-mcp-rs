# Documentation Style Guide

## Overview

This guide establishes documentation standards for the Prism MCP SDK to ensure consistency, clarity, and professionalism across all documentation.

## General Principles

### Writing Style
- Use clear, concise, academic writing
- Prefer technical accuracy over marketing language
- Write in present tense for current functionality
- Use active voice when possible
- Avoid colloquialisms and informal language

### Terminology

#### Standard Terms
- **Prism MCP SDK** - Full product name
- **prism-mcp-rs** - Crate name (lowercase, hyphenated)
- **Plugin** - Not "extension", "addon", or "module"
- **Component** - When referring to plugin-provided functionality
- **Tool** - Executable function (not "handler" in user docs)
- **Resource** - Data provider (not "endpoint" or "source")

#### Avoid
- "MCP Protocol SDK" (inconsistent naming)
- "Production-ready" in every section (state once if needed)
- Marketing superlatives ("revolutionary", "unmatched", etc.)

## Formatting Standards

### Headers
- Use ATX-style headers (#, ##, ###)
- Maintain consistent hierarchy
- No emojis or special characters in headers
- Use sentence case for headers

### Lists
- Use dash (-) for unordered lists
- Use numbers (1., 2., 3.) for ordered lists
- Maintain 2-space indentation for nested items
- Keep list items parallel in structure

### Code Blocks
```rust
// Always specify the language
// Include context where helpful
// Keep examples concise and focused
```

### Tables
| Column | Description |
|--------|-------------|
| Use pipe tables | With clear headers |
| Align columns | For readability |
| Keep descriptions | Concise |

## Cross-References

### Internal Links
- Use relative paths: `[Link Text](../path/to/doc.md)`
- Include section anchors: `[Link Text](doc.md#section)`
- Verify all links work before committing

### External Links
- Always use HTTPS when available
- Include link text that describes destination
- Avoid "click here" or "this link"

## Content Organization

### Document Structure
1. **Title** - Clear, descriptive H1 header
2. **Overview** - Brief introduction to the topic
3. **Related Documentation** - Cross-references (if applicable)
4. **Main Content** - Organized with clear sections
5. **Examples** - Practical, working examples
6. **References** - Links to additional resources

### Avoiding Redundancy
- Maintain single sources of truth
- Use cross-references instead of duplicating content
- Consolidate similar information into one location
- Update references when moving content

## Visual Elements

### Icons and Emojis
- **Do not use emojis** in documentation
- Use traditional software engineering notation
- Prefer text labels over icons
- Use standard markdown formatting for emphasis

### Badges
- Limit to essential status indicators
- Use standard shields.io badges
- Place at top of README only
- Keep badge count reasonable (< 20)

## Code Examples

### Requirements
- Must compile and run without errors
- Include all necessary imports
- Show both simple and complex usage
- Comment non-obvious code
- Use consistent naming conventions

### Format
```rust
use prism_mcp_rs::prelude::*;

// Descriptive comment about what this does
fn example_function() -> McpResult<()> {
    // Implementation
    Ok(())
}
```

## API Documentation

### Doc Comments
- Use `///` for public items
- Include brief description first
- Add `# Examples` section for complex items
- Document all parameters and return values
- Include `# Errors` section when applicable

### Example
```rust
/// Processes incoming MCP requests.
///
/// This function handles all request types defined in the MCP specification
/// and routes them to appropriate handlers.
///
/// # Arguments
/// * `request` - The incoming MCP request
///
/// # Returns
/// * `Ok(Response)` - Successfully processed request
/// * `Err(McpError)` - Processing failed
///
/// # Examples
/// ```
/// let response = process_request(request)?;
/// ```
pub fn process_request(request: Request) -> McpResult<Response> {
    // Implementation
}
```

## File Naming

### Documentation Files
- Use UPPERCASE for guides: `CONTRIBUTING.md`, `PLUGIN_GUIDE.md`
- Use lowercase for references: `api-reference.md`
- Use descriptive names: `plugin-types.md` not `types.md`

### Consistency Rules
- README.md is always uppercase
- License files are uppercase: LICENSE, LICENSE-MIT
- Config files are lowercase: cargo.toml, rustfmt.toml

## Review Checklist

Before submitting documentation:

- [ ] No emojis or special characters
- [ ] Consistent terminology throughout
- [ ] All cross-references work
- [ ] No duplicate information
- [ ] Code examples compile
- [ ] Headers follow hierarchy
- [ ] Lists are properly formatted
- [ ] Tables are aligned
- [ ] File names follow conventions
- [ ] Writing is clear and concise

## Enforcement

This style guide should be:
- Referenced in CONTRIBUTING.md
- Checked during PR reviews
- Applied to all new documentation
- Used to update existing documentation gradually

## Updates

This guide is a living document. Propose changes through:
1. GitHub issues for discussion
2. Pull requests for specific changes
3. Team consensus before major changes

---

Last updated: 2025-01-09
Version: 1.0.0
