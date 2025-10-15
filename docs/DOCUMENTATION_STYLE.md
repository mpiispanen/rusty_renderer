# Documentation Style Guide

This guide ensures consistent formatting across all project documentation, especially for conversion to HTML.

## Code Blocks

### ✅ Correct Format

Always put code blocks on their own lines with blank lines before and after:

```markdown
Some text here.

\`\`\`rust
pub fn example() {
    println!("Hello");
}
\`\`\`

More text here.
```

### ❌ Incorrect Format

Don't put code blocks immediately after headers or text without blank lines:

```markdown
Some text here.
\`\`\`rust
pub fn bad_example() {
    // This won't render properly in HTML
}
\`\`\`
More text here.
```

## Headers

### Spacing
- Always use blank lines before and after headers
- H4 headers (####) should have blank line before

```markdown
Some content.

#### Section Header

Content continues.
```

### Hierarchy
- H1 (#): Document title only
- H2 (##): Major sections
- H3 (###): Subsections
- H4 (####): Detailed items

## Lists

### Ordered Lists
```markdown
1. First item
2. Second item
3. Third item
```

### Unordered Lists
```markdown
- First item
- Second item
- Third item
```

### Nested Lists
Use 2 spaces for indentation:

```markdown
- Parent item
  - Child item
  - Another child
- Another parent
```

## Inline Code

Use backticks for:
- Function names: `initialize()`
- Variable names: `backend_type`
- File paths: `src/main.rs`
- Short code snippets: `Result<()>`

```markdown
The `GraphicsBackend` trait provides the main interface.
```

## Emphasis

- **Bold** for emphasis: `**bold text**`
- *Italic* for terms: `*term*`
- ~~Strikethrough~~ for deprecated: `~~deprecated~~`

## Links

### Internal Links
```markdown
See [Design Document](DESIGN.md) for details.
```

### External Links
```markdown
Learn more at [Rust Book](https://doc.rust-lang.org/book/)
```

## Tables

Use proper markdown table syntax:

```markdown
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Data 1   | Data 2   | Data 3   |
| Data 4   | Data 5   | Data 6   |
```

## Code Examples

### Short Examples (< 10 lines)
Use inline code blocks with language specification:

```markdown
\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`
```

### Long Examples
Consider linking to actual source files:

```markdown
See the implementation in [`src/backends/mod.rs`](../src/backends/mod.rs).
```

## Checkboxes (Task Lists)

```markdown
- [ ] Incomplete task
- [x] Completed task
```

## Blockquotes

Use for important notes or warnings:

```markdown
> **Note:** This is an important callout.

> **Warning:** This operation is dangerous.
```

## HTML Conversion Considerations

### Code Block Issues
The `convert.sh` script uses `lowdown` which requires:
1. Blank line before code block opening
2. Blank line after code block closing
3. Language identifier on same line as opening backticks

### Character Escaping
These characters are automatically escaped in HTML:
- `<` → `&lt;`
- `>` → `&gt;`
- `&` → `&amp;`

No manual escaping needed in markdown.

## File Organization

### Document Headers
Every document should start with:

```markdown
# Document Title

**Brief description of the document**

**Last Updated:** YYYY-MM-DD
```

### Table of Contents
For long documents (> 5 sections), include a TOC:

```markdown
## Table of Contents

- [Section 1](#section-1)
- [Section 2](#section-2)
- [Section 3](#section-3)
```

## Common Mistakes to Avoid

1. ❌ Code block directly after header without blank line
2. ❌ Missing language identifier on code blocks
3. ❌ No blank lines around code blocks
4. ❌ Inconsistent indentation in lists
5. ❌ Missing blank line before headers

## Testing HTML Output

After making documentation changes:

```bash
# Convert to HTML
./docs/convert.sh

# Check specific file
cat docs/html/YOUR_FILE.html | grep -A 5 "code block pattern"
```

## Examples

### Planning Document Template

```markdown
# Milestone X Planning

**Date:** YYYY-MM-DD  
**Milestone:** MX - Milestone Name  
**Status:** Planning

## Overview

Brief description of the milestone.

## Core Components

### Component Name

**Purpose:** What this component does

\`\`\`rust
pub trait ComponentName {
    fn method(&self) -> Result<()>;
}
\`\`\`

**Key Features:**
- Feature 1
- Feature 2
```

### Retrospective Template

```markdown
# Milestone X Retrospective

**Date:** YYYY-MM-DD  
**Milestone:** MX - Milestone Name  
**Status:** Complete

## Overview

Summary of what was accomplished.

## What Went Well ✅

- Item 1
- Item 2

## Challenges & Solutions 💡

1. **Challenge:** Description
   - **Solution:** How it was resolved
```

## Validation Checklist

Before committing documentation:

- [ ] All code blocks have blank lines before/after
- [ ] Language identifiers on code blocks (rust, bash, toml, etc.)
- [ ] Headers have proper spacing
- [ ] Links are working (for internal links)
- [ ] Run `./docs/convert.sh` to verify HTML output
- [ ] Check no weird formatting in generated HTML

## Tools

- **Editor:** Any markdown editor
- **Converter:** `lowdown` (via `./docs/convert.sh`)
- **Preview:** Check `docs/html/*.html` files
- **Validation:** Visual inspection of HTML output

---

**Remember:** Good markdown formatting ensures good HTML output!
